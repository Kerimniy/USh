
let webp_dl_btn = document.getElementById("webp-download-button")
let jpeg_dl_btn = document.getElementById("jpeg-download-button")
let png_dl_btn = document.getElementById("png-download-button")
let dl_bg = document.getElementById("download-menu-bg")
let dl_menu = document.getElementById("dl-menu")

webp_dl_btn.addEventListener("click",()=>{
    downloadQR("webp")
})
jpeg_dl_btn.addEventListener("click",()=>{
    downloadQR("jpeg")
})
png_dl_btn.addEventListener("click",()=>{
    downloadQR("png")
})

dl_bg.addEventListener("click",()=>{
    dl_menu.classList.remove("open")
})

document.getElementById("download-qr").addEventListener("click",()=>{
    dl_menu.classList.add("open")
})


function downloadQR(type){
    let cnvs = document.getElementById("qrcode").querySelector("canvas")

    let dataURL
    if (type==="webp") {
        dataURL = cnvs.toDataURL("image/webp");
    }
    else if (type==="png") {
        dataURL = cnvs.toDataURL("image/png");
    }
    else if (type==="jpeg") {
        dataURL = cnvs.toDataURL("image/jpeg");
    }

    const arr = dataURL.split(',');
    const mime = arr[0].match(/:(.*?);/)[1];
    const bstr = atob(arr[1]);

    let n = bstr.length;
    const u8arr = new Uint8Array(n);

    while (n--) {
        u8arr[n] = bstr.charCodeAt(n);
    }

    const blob = new Blob([u8arr], { type: mime });

    const url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = url;
    let name="qrcode"
    if (link.innerText!==undefined){
        name=link.innerText
    }
    a.download = `${name}.${type}`;
    a.click();

    URL.revokeObjectURL(url);
}
