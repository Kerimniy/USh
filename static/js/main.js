    
let input = document.getElementById("input")
let shorten0 = document.getElementById("shorten0")
let shorten1 = document.getElementById("shorten1")


let link = document.getElementById("link")
link.addEventListener("click",()=>{
    navigator.clipboard.writeText(link.innerText)
})
let edit_qr = document.getElementById("edit-qr")
edit_qr.addEventListener("click",()=>{
    document.querySelector(".edit-qr-container").classList.toggle("open")
})

function registerUrl(){
    let url = input.value

    link.dataset.url = input.value

    shorten0.classList.remove("active")
    shorten1.classList.remove("active")

    let data = {
        "url":url
    }

    fetch("/-/create/",  {method: 'POST',headers: {'Content-Type': 'application/json'}, body: JSON.stringify(data)})
    .then(resp => { 

        if (!resp.ok){
            input.classList.add("error")
            document.querySelector(".status-container").classList.add("error")


            if (resp.status==429){
                 document.getElementById("status-text").innerText="Error 429. Too many requests. Please wait."
            }
            else{
                 document.getElementById("status-text").innerText=`Unexpexted error. (${resp.status}).`
            }

        }
        else{
            input.classList.remove("error")
            document.querySelector(".status-container").classList.remove("error")
            document.querySelector(".result-container").classList.add("open")

        }

        return resp.json()
            })
            .then(res => {
                const url = new URL(res.url, window.location.origin);
                updateCode(url.href)
                link.innerText=url.href

                window.history.pushState('', '', `?source=${input.value}&link=${url.href}`);

            })

    
}

shorten0.addEventListener("click", ()=>{
    if (shorten0.classList.contains("active")){
registerUrl()
    }
})
shorten1.addEventListener("click", ()=>{
    if (shorten1.classList.contains("active")){
registerUrl()
    }
})

    
    
  
function isValidHttpUrl(string) {
    let url;

    if (string===link.dataset.url){
        return false
    }
    
    try {
        url = new URL(string);
    } catch (_) {
        return false;
    }
    let splitted = string.split(".")
    if (splitted.length<2){
        return false
    }

    return url.protocol === "http:" || url.protocol === "https:";
}


    



input.addEventListener("input",()=>{
    if (isValidHttpUrl(input.value)){
        shorten0.classList.add("active")
        shorten1.classList.add("active")
    }
    else{
        shorten0.classList.remove("active")
        shorten1.classList.remove("active")
    }
})

    

    
const dropZone = document.getElementById('drop-zone');
const fileInput = document.getElementById('file-input');
let img = document.getElementById("qrcode").querySelector("img")


    let new_src = ""

    dropZone.addEventListener('click', () => fileInput.click());

    fileInput.addEventListener('change', () => {
if (fileInput.files.length) {
    handleImage(fileInput.files[0]);
}
    });

    dropZone.addEventListener('dragover', (e) => {
e.preventDefault();
dropZone.classList.add('drop-zone--over');
    });

    ['dragleave', 'dragend'].forEach(type => {
dropZone.addEventListener(type, () => {
    dropZone.classList.remove('drop-zone--over');
});
    });

    dropZone.addEventListener('drop', (e) => {
e.preventDefault();
if (e.dataTransfer.files.length) {
    handleImage(e.dataTransfer.files[0]);
}
dropZone.classList.remove('drop-zone--over');
    });

    function handleImage(file) {
        if (!file.type.match('image.*')) {
            dropZone.querySelector('.drop-zone__prompt').textContent = "Only PNG, JPG, WEBP";
            dropZone.classList.add("red")
            return;
}
dropZone.classList.remove("red")

const reader = new FileReader();
reader.onload = (event) => {
    new_src = event.target.result;
    dropZone.querySelector('.drop-zone__prompt').textContent = file.name;
};
reader.readAsDataURL(file);
    }
    
    let main_color = document.getElementById("main-color")
    let bg_color = document.getElementById("bg-color")

    function resetQR(){
        main_color.value="#000000"
        bg_color.value="#ffffff"
        main_color.parentElement.querySelector('input[type="text"]').value="000000";
        bg_color.parentElement.querySelector('input[type="text"]').value="ffffff";
        new_src=""
        dropZone.querySelector('.drop-zone__prompt').textContent = "";
        dropZone.classList.remove("red")
    }


    function updateCode(url){
        if (url===undefined){
            url=link.innerText
        }
        document.getElementById("qrcode").innerHTML=""
        document.getElementById("qrcode").style.opacity="0"
        var qrcode = new QRCode(document.getElementById("qrcode"), {
        text: url,
        width: 512,
        height: 512,
        colorDark : main_color.value,
        colorLight : bg_color.value,
        correctLevel : QRCode.CorrectLevel.H
            });

            if (new_src!=""){

        let cnvs = document.getElementById("qrcode").querySelector("canvas")
        context = cnvs.getContext('2d');
        base_image = new Image();
        base_image.crossOrigin = "anonymous";

        base_image.src=new_src
        base_image.onload = function(){


            let w=150
            let h=150

            if (base_image.height>base_image.width){
        w=base_image.width/(base_image.height/150)
            }
            else{
        h=base_image.height/(base_image.width/150)
            }

            context.drawImage(base_image,256-75, 256-75,w,h);

            let dataURL = cnvs.toDataURL("image/webp");

            img.src = dataURL
            img.style.width="100%"


        }
            }
            let img = document.getElementById("qrcode").querySelector("img")
            img.style.width="100%"
            document.getElementById("qrcode").style.opacity="1"
        }



        const textInputs = document.querySelectorAll('input.color');

        textInputs.forEach(textInput => {
            const colorPicker = textInput.parentElement.parentElement.querySelector('input[type="color"]');

            if (!colorPicker) return;

            colorPicker.addEventListener('input', () => {
        textInput.value = colorPicker.value.substring(1);
            });

            textInput.addEventListener('input', () => {
        let val = textInput.value;

        if (val.length === 6) {
            colorPicker.value = '#' + val;
        }
            });

            textInput.addEventListener('keypress', (e) => {
        if (!/[0-9a-fA-F]/.test(e.key)) {
            e.preventDefault();
        }
            });
    });
updateCode()


