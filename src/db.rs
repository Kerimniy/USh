use std::ops::Deref;
use async_rusqlite;
use async_rusqlite::{rusqlite, Connection};
use async_rusqlite::rusqlite::types::ValueRef;
use bcrypt::{hash, verify, DEFAULT_COST};

use tokio::sync::Mutex;
use std::sync::Arc;
use async_rusqlite::rusqlite::fallible_streaming_iterator::FallibleStreamingIterator;
use async_rusqlite::rusqlite::{params, Error};
use async_rusqlite::rusqlite::types::Type::Null;
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;

static CONN: OnceCell<Connection> = OnceCell::const_new();


pub async fn get_conn() -> &'static Connection {
    CONN.get_or_init(|| async {
        Connection::open("db/main.db").await.unwrap()
    }).await
}


pub async fn init_db() -> usize{

    let conn = get_conn().await;

    conn.call(|conn| {
        conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA temp_store = MEMORY;
        PRAGMA cache_size = -10000;
    ")
    }).await.unwrap();
    

    match conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS links (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            link TEXT NOT NULL UNIQUE,
            source TEXT NOT NULL
        )",
            (),
        )
    }).await{
        Ok(_) =>0,
        Err(e)=> {println!("{:?}",e); return 2},
    };


    match conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (
            login TEXT PRIMARY KEY NOT NULL,
            password TEXT NOT NULL
        )",
            (),
        )
    }).await{
        Ok(_) =>0,
        Err(e)=> {println!("{:?}",e); return 3},
    };

    /*
    match conn.call(|conn| {
        conn.execute(
            "INSERT INTO count (id)
                SELECT 0
                WHERE NOT EXISTS (
                    SELECT 1 FROM count
                );",
            (),
        )
    }).await{
        Ok(_) =>0,
        Err(err)=> {println!("{:?}",err); return 4},
    };
    */

    0

}


pub async fn get_count() -> Result<u64, async_rusqlite::rusqlite::Error>{
    let conn = get_conn().await;


    let q:Result<u64, async_rusqlite::rusqlite::Error>= conn.call(|conn| {

        Ok(conn.last_insert_rowid() as u64)


    }).await;

    match q {
        Ok(q)=>{
            if q==0u64{
                let count_alt = match get_count_alt().await{
                    Ok(count)=>{count},
                    Err(e)=>{return  Err(e)}
                };

                return Ok(count_alt)

            }
            else{
                return Ok(q)
            }
        },
        Err(e)=>{Err(e)}
    }


}


async fn get_count_alt() -> Result<u64, async_rusqlite::rusqlite::Error>{
    let conn = get_conn().await;


    let q:Result<u64, async_rusqlite::rusqlite::Error>= conn.call(|conn| {
        let mut stmt = conn.prepare("SELECT MAX(id) FROM links")?;
        let mut rows = stmt.query([])?;

        let mut res: usize = 0;
        while let Some(row) = rows.next()? {
            res = match row.get(0){
                Ok(t)=>t,
                Err(e)=>{
                    match e {
                        async_rusqlite::rusqlite::Error::InvalidColumnType(_, _, Null) => {
                            return Ok(0);
                        }
                        _ => return Err(e),
                    }
                }
            }
        }

        Ok(res as u64)

    }).await;

    q
}


pub async fn is_user_exists() -> Result<bool, async_rusqlite::rusqlite::Error>{
    let uc = match get_user_count().await{
        Ok(uc)=>uc,
        Err(e)=>{return Err(e)}
    };
    return Ok(uc > 0);
}


async fn get_user_count()-> Result<usize, async_rusqlite::rusqlite::Error>{
    let conn = get_conn().await;


    let q:Result<usize, async_rusqlite::rusqlite::Error>= conn.call(|conn| {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM user")?;
        let mut rows = stmt.query([])?;

        let mut res: usize = 0usize;
        while let Some(row) = rows.next()?{
            res = row.get(0)?
        }



        Ok(res)

    }).await;
    q
}


pub async fn resolve_link(link: String) -> Result<String, async_rusqlite::rusqlite::Error>{
    let conn = get_conn().await;


    let q:Result<String, async_rusqlite::rusqlite::Error>= conn.call(|conn| {
        let mut stmt = conn.prepare("SELECT source FROM links WHERE link=?")?;
        let mut rows = stmt.query([link])?;

        let mut res: String = String::new();
        while let Some(row) = rows.next()? {
            res = row.get(0)?
        }

        if res == String::new(){
            return Err(async_rusqlite::rusqlite::Error::QueryReturnedNoRows)
        }

        Ok(res)

    }).await;

    q
}


pub async fn register_link(link: String, source: String) -> Result<usize,async_rusqlite::rusqlite::Error>{

    let conn = get_conn().await;


    let q =
        conn.call(move |conn| {
            /*
            let ex = conn.execute(
                "UPDATE count SET id=id+1",
                (),
            );

            match ex {
                Ok(_) =>{}
                Err(e)=>{ return Err(e); }
            }
            */

            conn.execute(
                "INSERT OR IGNORE INTO links(link, source) VALUES (?1, ?2)",
                (link, source),
            )


        }).await;
    q



}


pub async fn create_user(login: String, password: String) -> Result<usize,async_rusqlite::rusqlite::Error>{

    let conn = get_conn().await;


    let q =
        conn.call(move |conn| {

            let password = match hash(password,DEFAULT_COST){
                Ok(password)=>password,
                Err(_)=>{return Err(rusqlite::Error::UnwindingPanic)}
            };

            conn.execute(
                "INSERT OR IGNORE INTO user(login, password) VALUES (?1, ?2)",
                (login, password),
            )


        }).await;
    q



}


pub async fn check_user(login: String, password: String) -> Result<bool, async_rusqlite::rusqlite::Error>{
    let conn = get_conn().await;


    let q:Result<String, async_rusqlite::rusqlite::Error>= conn.call(|conn| {
        let mut stmt = conn.prepare("SELECT password FROM user WHERE login=?")?;
        let mut rows = stmt.query([login])?;

        let mut res: String = String::new();
        while let Some(row) = rows.next()? {
            res = row.get(0)?
        }

        if res == String::new(){
            return Err(async_rusqlite::rusqlite::Error::QueryReturnedNoRows)
        }

        Ok(res)

    }).await;

    let password_hash = match q{
        Ok(hash)=>hash,
        Err(e)=>{return Err(e)}
    };

    match verify(&password, &password_hash){
        Ok(res)=>{return Ok(res)},
        Err(e)=>{return Err(async_rusqlite::rusqlite::Error::UnwindingPanic)}
    }
}


pub async fn exequte(query: String) -> Result<usize,async_rusqlite::rusqlite::Error>{

    let conn = get_conn().await;


    let q =
        conn.call(move |conn| {

            conn.execute(
                &query,
                (),
            )


        }).await;
    q



}


pub async fn get_query(query: String) -> Result<Vec<String>,async_rusqlite::rusqlite::Error>{

    let conn = get_conn().await;


    let q:Result<Vec<String>, async_rusqlite::rusqlite::Error>= conn.call(move |conn| {
        let mut stmt = conn.prepare(&query)?;

        let count = stmt.column_count();

        if count >0 {
            let mut rows = stmt.query([])?;



            let mut res: Vec<String> = Vec::new();
            while let Some(row) = rows.next()? {
                for i in 0..count {
                    let v = match row.get(i){
                        Ok(v)=>value_to_string(v),
                        Err(e)=>{println!("{:?}",e); return Err(e)}
                    };
                    res.push(v);
                }
                res.push("\n".to_string());
            }

            Ok(res)

        }
        else {
            match stmt.execute(params![]){
                Ok(_) => Ok(Vec::new()),
                Err(e) => Err(e)
            }
        }
    }).await;
    q



}

fn value_to_string(v: async_rusqlite::rusqlite::types::Value) -> String {
    match v {
        async_rusqlite::rusqlite::types::Value::Null => "NULL".into(),
        async_rusqlite::rusqlite::types::Value::Integer(i) => i.to_string(),
        async_rusqlite::rusqlite::types::Value::Real(f) => f.to_string(),
        async_rusqlite::rusqlite::types::Value::Text(t) => t,
        async_rusqlite::rusqlite::types::Value::Blob(_) => "<blob>".into(),
    }
}