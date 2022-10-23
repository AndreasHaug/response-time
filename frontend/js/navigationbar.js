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


async function setMinutes(e, m) {
    m.target.classList.add("clicked");
    if (prevMinutes !== null) {
	prevMinutes.classList.remove("clicked");
    }
    prevMinutes = m.target;
    cost = e;
    await draw();
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

let button1min = document.getElementById("1min");
button1min.classList.add("clicked");

let button2min = document.getElementById("2min");
let button5min = document.getElementById("5min");
let button10min = document.getElementById("10min");
let button20min = document.getElementById("20min");

button1min.addEventListener("click", (m) => { setMinutes(1, m);  });
button2min.addEventListener("click", (m) => { setMinutes(2, m);  });
button5min.addEventListener("click", (m) => { setMinutes(5, m);  });
button10min.addEventListener("click", (m) => { setMinutes(10, m);  });
button20min.addEventListener("click", (m) => { setMinutes(20, m);  });

let prevMinutes = button1min;
