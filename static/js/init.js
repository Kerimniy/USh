const queryString = window.location.search;
const urlParams = new URLSearchParams(queryString);

document.getElementById("input").value = urlParams.get("source")
document.getElementById("link").innerText = urlParams.get("link")

if (document.getElementById("link").innerText.trim()!="" && document.getElementById("input").value.trim()!=""){

    updateCode(urlParams.get("link"))
    input.classList.remove("error")
    document.querySelector(".status-container").classList.remove("error")
    document.querySelector(".result-container").classList.add("open")

}
