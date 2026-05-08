/*
    BSD 3-Clause License
    
    Copyright (c) 2026, Kerimniy
    
    Redistribution and use in source and binary forms, with or without
    modification, are permitted provided that the following conditions are met:
    
    1. Redistributions of source code must retain the above copyright notice, this
       list of conditions and the following disclaimer.
    
    2. Redistributions in binary form must reproduce the above copyright notice,
       this list of conditions and the following disclaimer in the documentation
       and/or other materials provided with the distribution.
    
    3. Neither the name of the copyright holder nor the names of its
       contributors may be used to endorse or promote products derived from
       this software without specific prior written permission.
    
    THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
    AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
    IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
    DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
    FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
    DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
    SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
    CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
    OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
    OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
 

mod db;

use tokio::sync::OnceCell;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use std::net::SocketAddr;
use async_rusqlite::Connection;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

use rand::{Rng, RngExt};
use tokio;
use axum::{routing::get, extract::Json, routing::post, handler::Handler, Router, http::StatusCode, Extension};
use axum::extract::Path;
use axum::response::{Html, IntoResponse, Redirect};
use serde::{Deserialize, Serialize};

use tera::{Tera, Context};

use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
};

use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use once_cell::sync::Lazy;

use cookie::{Key, SameSite};
use rand::random;
use rusqlite::fallible_iterator::FallibleIterator;
use crate::db::exequte;


struct AppState {
    admin_exists: AtomicBool,
}


static TERA: OnceCell<Tera> = OnceCell::const_new();


#[derive(Deserialize, Serialize)]
struct CreateLink {
    url: String,
}


#[derive(Deserialize, Serialize)]
struct SQL_Query {
    query: String,
}


#[derive(Deserialize, Serialize)]
struct Userdata {
    login: String,
    password: String,
}


pub const BASE64_CHARS: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
    'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    '(', ')',
];

static SECRET_KEY: Lazy<cookie::Key> = Lazy::new(||Key::from(&read_secret_key(".SECRETKEY")));
static HOST: Lazy<String> = Lazy::new(|| {std::fs::read_to_string("HOST").unwrap_or(String::from("127.0.0.1:8000"))});


pub async fn get_tera() -> &'static Tera {
    TERA.get_or_init(|| async {
        Tera::new("templates/**/*").unwrap()
    }).await
}


async fn create_link(Json(payload): Json<CreateLink>) -> impl IntoResponse {

    let link = gen_link(1);

    (
        StatusCode::CREATED,
        Json(CreateLink {
            url: link,
        })
    )
}


async fn resolve_link(Path(link): Path<String>) -> impl IntoResponse {
    let resolved = match db::resolve_link(link).await{
        Ok(r) => r,
        Err(e) => {
            if e==async_rusqlite::rusqlite::Error::QueryReturnedNoRows{
                let tera = get_tera().await;

                let context = Context::new();
                let rendered = match tera.render("404.html", &context){
                    Ok(r)=>{return (StatusCode::NOT_FOUND,Html(r)).into_response()},
                    Err(_)=>{return (StatusCode::INTERNAL_SERVER_ERROR).into_response()}
                };

            }
            else {
                return (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    };

    (StatusCode::FOUND, Redirect::temporary(&resolved)).into_response()
}


async fn gen_link_handle(Json(payload): Json<CreateLink>) -> impl IntoResponse {

    let num: u64 = match db::get_count().await{
        Ok(n)=>n,
        Err(e)=>{println!("{:?}",e); return (StatusCode::INTERNAL_SERVER_ERROR).into_response()}
    };
    let link = gen_link(num);
    match db::register_link(link.clone(),payload.url).await{
        Ok(_)=>{}
        Err(e)=>{println!("{:?}",e);return (StatusCode::INTERNAL_SERVER_ERROR).into_response()}
    };

    (StatusCode::CREATED,
     Json(CreateLink {
         url: link,
     })).into_response()

}


async fn render_index(Extension(state):Extension<Arc<AppState>>) -> impl IntoResponse {

    let admin_exists = state.admin_exists.load(Ordering::Relaxed);

    if admin_exists==false{
        return Redirect::temporary("/-/reg/").into_response()
    }

    let tera = get_tera().await;

    let context = Context::new();

    let st: StatusCode;

    let rendered = match tera.render("index.html", &context){
        Ok(r)=>{st=StatusCode::OK; r},
        Err(_)=>{st=StatusCode::INTERNAL_SERVER_ERROR;"500".to_string()}
    };

    return (st, Html(rendered)).into_response()
}


async fn render_debug(Extension(state):Extension<Arc<AppState>>,cookies: Cookies) -> impl IntoResponse {

    let admin_exists = state.admin_exists.load(Ordering::Relaxed);

    if admin_exists==false{
        return Redirect::temporary("/-/reg/").into_response()
    }

    if is_login(&cookies)==false{
        return Redirect::temporary("/-/login/").into_response()
    }

    let tera = get_tera().await;

    let context = Context::new();

    let st: StatusCode;

    let rendered = match tera.render("debug.html", &context){
        Ok(r)=>{st=StatusCode::OK; r},
        Err(_)=>{st=StatusCode::INTERNAL_SERVER_ERROR;"500".to_string()}
    };

    return (st, Html(rendered)).into_response()
}


async fn render_reg(Extension(state):Extension<Arc<AppState>>) -> impl IntoResponse {

    let admin_exists = state.admin_exists.load(Ordering::Relaxed);

    if admin_exists==true{
        return Redirect::temporary("/-/login/").into_response()
    }

    let tera = get_tera().await;

    let context = Context::new();

    let st: StatusCode;

    let rendered = match tera.render("reg.html", &context){
        Ok(r)=>{st=StatusCode::OK; r},
        Err(_)=>{st=StatusCode::INTERNAL_SERVER_ERROR;"500".to_string()}
    };

    return (st, Html(rendered)).into_response()
}


async fn render_login(Extension(state):Extension<Arc<AppState>>) -> impl IntoResponse {

    let admin_exists = state.admin_exists.load(Ordering::Relaxed);

    if admin_exists==false{
        return Redirect::temporary("/-/reg/").into_response()
    }

    let tera = get_tera().await;

    let context = Context::new();

    let st: StatusCode;

    let rendered = match tera.render("login.html", &context){
        Ok(r)=>{st=StatusCode::OK; r},
        Err(_)=>{st=StatusCode::INTERNAL_SERVER_ERROR;"500".to_string()}
    };

    return (st, Html(rendered)).into_response()
}


async fn reg(Extension(state):Extension<Arc<AppState>>,cookies: Cookies, Json(data): Json<Userdata>) -> impl IntoResponse{
    let admin_exists = state.admin_exists.load(Ordering::Relaxed);
    if admin_exists==true{
        return (StatusCode::BAD_REQUEST).into_response();
    }

    match db::create_user(data.login,data.password).await{
        Ok(r)=>{add_signed_cookie(&cookies); state.admin_exists.store(true,Relaxed); return (StatusCode::OK).into_response()}
        Err(e)=>{return (StatusCode::INTERNAL_SERVER_ERROR).into_response()}
    };

}


async fn login(Extension(state):Extension<Arc<AppState>>,cookies: Cookies, Json(data): Json<Userdata>) -> impl IntoResponse{
    let admin_exists = state.admin_exists.load(Ordering::Relaxed);
    if admin_exists==false{
        return (StatusCode::BAD_REQUEST).into_response();
    }



    match db::check_user(data.login,data.password).await{
        Ok(r)=>{add_signed_cookie(&cookies); return (StatusCode::OK).into_response()}
        Err(e)=>{return (StatusCode::INTERNAL_SERVER_ERROR).into_response()}
    };

}


async fn logout(cookies: Cookies) -> impl IntoResponse{

    remove_signed_cookie(&cookies);

    Redirect::temporary("/").into_response()

}


async fn execute_command(Extension(state):Extension<Arc<AppState>>,cookies: Cookies,Json(SQL_Query): Json<SQL_Query>)-> impl IntoResponse{
    let admin_exists = state.admin_exists.load(Ordering::Relaxed);
    if admin_exists==false{
        return (StatusCode::BAD_REQUEST).into_response();

    };

    if is_login(&cookies)==false{
        return (StatusCode::UNAUTHORIZED).into_response();
    }

    match db::get_query(SQL_Query.query).await{
        Ok(r)=>{
            let _str: String = r.join(" ");
            return _str.into_response()
        }
        Err(e)=>{
            return {println!("{:?}",e); (StatusCode::INTERNAL_SERVER_ERROR).into_response()}
        }
    }

}

async fn not_found_handler() -> impl IntoResponse {
    let tera = get_tera().await;

    let context = Context::new();
    match tera.render("404.html", &context){
        Ok(r)=>{return (StatusCode::NOT_FOUND,Html(r)).into_response()},
        Err(_)=>{return (StatusCode::INTERNAL_SERVER_ERROR).into_response()}
    };
}

#[tokio::main]
async fn main() {

    db::init_db().await;


    let admin_exist = db::is_user_exists().await.unwrap();

    let state = Arc::new(AppState {
        admin_exists: AtomicBool::new(admin_exist),
    });

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(5)
        .finish()
        .unwrap();

    let app = Router::new()
        .route("/-/create/", post(gen_link_handle)).layer(GovernorLayer::new(governor_conf))
        .route("/", get(render_index))
        .route("/-/reg/", get(render_reg))
        .route("/-/reg", get(render_reg))
        .route("/-/debug/", get(render_debug))
        .route("/-/login/", get(render_login))
        .route("/-/login", get(render_login))
        .route("/-/logout/", get(logout))
        .route("/-/logout", get(logout))
        .route("/-/api/reg", post(reg))
        .route("/-/api/login", post(login))
        .route("/-/api/sql-debug/", post(execute_command))
        .route("/{link}", get(resolve_link))
        .nest_service("/static", ServeDir::new("static"))
        .layer(Extension(state))
        .layer(CookieManagerLayer::new())
        .fallback(not_found_handler);

    let host = HOST.clone();
    println!("Starting server on {}", &host);
    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}


fn gen_link(num: u64) -> String {
    let mut rng = rand::rng();

    let salt = rng.random_range(1..=99);

    let mut num: u64 = num*100+salt;

    let mut res: Vec<char> = Vec::new();


    while num>63 {
        res.push(BASE64_CHARS[(num%64) as usize]);
        num/=64;
    }
    res.push(BASE64_CHARS[(num%64) as usize]);

    res.into_iter().collect::<String>()
}


fn add_signed_cookie(cookies: &Cookies){
    let mut  cookie = tower_cookies::Cookie::new("session","admin".to_string());
    cookie.make_permanent();
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");

    cookies.signed(&SECRET_KEY).add(cookie);
}


fn is_login(cookies: &Cookies)->bool{

    let cookie = cookies.signed(&SECRET_KEY).get("session");

    return cookie.is_some();
}


fn remove_signed_cookie(cookies: &Cookies) {
    let mut  cookie = tower_cookies::Cookie::new("session","admin".to_string());
    cookie.make_permanent();
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");

    cookies.signed(&SECRET_KEY).remove(cookie);
}


fn read_secret_key(path: &str) -> [u8;64]{
    let content =match std::fs::read(path){
        Ok(string_content) => string_content,
        Err(_) => {

            let key: [u8;64] = random();
            std::fs::write(".SECRETKEY", key);
            return key;
            key.to_vec()
        }
    };

    let mut key:[u8;64] = [0;64];

    if content.len() !=64{
        key = random();
        std::fs::write(".SECRETKEY", key);

    }
    else {
        for i in 0..64{
            key[i] = content[i];
        }
    }

    key
}