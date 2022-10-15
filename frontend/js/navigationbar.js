function change(e) { 
    e.target.classList.add("clicked"); 
    if (prev !== null) { 
 	prev.classList.remove("clicked"); 
    } 
    prev = e.target;
}


function changePolygon(e) {
    change(e);
    setPolygon();
    
}


function changeLinestrings(e) {
    change(e);
    setLinestrings();
}


function changeBoth(e) {
    change(e);
    setBoth()
}


let polygonbutton = document.getElementById("polbutton"); 
let linebutton = document.getElementById("linbutton"); 
let bothbutton = document.getElementById("bothbutton");
let timebutton = document.getElementById("timebutton");
polygonbutton.classList.add("clicked") 
let prev = polygonbutton; 
polygonbutton.addEventListener("click", changePolygon); 
linebutton.addEventListener("click", changeLinestrings); 
bothbutton.addEventListener("click", changeBoth); 
// timebutton.addEventListener("click", )
