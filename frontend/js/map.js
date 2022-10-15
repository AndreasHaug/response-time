var map = L.map('map').setView([59.94, 10.75], 13.0);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
}).addTo(map);


const drawType = {
    P : 1,
    LS : 2,
    B : 3,
};


let circleOptions = {
    radius: 4,
    color: 'blue',
    fillColor: 'blue',
    fillOpacity: 1
}


let drawTypeValue = drawType.P;
let lines = [];
let polygon;
let marker = [];
let latlng;
let cost = 3;

let polJson = null;
let linJson = null;


map.on("click", (e) => {
    async function start() {
	latlng = e.latlng;
	// resetAll();
	await draw();
    }
    start();
})


async function setPolygon() {
    if (drawTypeValue === drawType.P) {
	return;
    }    
    drawTypeValue = drawType.P;
    drawExisting();
}


async function setLinestrings() {
    if (drawTypeValue === drawType.LS) {
	return;
    }
    drawTypeValue = drawType.LS;
    drawExisting();
}


async function setBoth() {
    if (drawTypeValue === drawType.B) {
	return;
    }    
    drawTypeValue = drawType.B;
    drawExisting();
}


async function draw() {
    resetAll();
    drawExisting();
}


async function drawExisting() {
    resetFigures();
    if (drawTypeValue === drawType.P) {
	drawPolygon();
    }
    else if (drawTypeValue === drawType.LS) {
	drawLinestrings();
    }
    else {
	drawBoth();
    }    
}

async function drawPolygon() {    
    if (latlng === undefined) {
	return;
    }
    if (polJson === undefined || polJson === null) {
	await getPolygonResponse();
    }
    polygon = L.polygon(polJson["results"]["PolygonResult"]["coordinates"],
			{color: 'blue'}).addTo(map);
    marker.push(L.circleMarker(polJson["start"]["coordinates"],
			       circleOptions).addTo(map));
}


async function drawLinestrings() {
    if (latlng === undefined) {
	return;
    }
    if (linJson === undefined || linJson === null) {
	await getLinestringResponse();
    }
    for (let a in linJson["results"]["MultilinestringResult"]["coordinates"]) {
	lines.push(L.polyline(linJson["results"]["MultilinestringResult"]["coordinates"][a],
			      {color: 'red'}).addTo(map));
    }
    marker.push(L.circleMarker(linJson["start"]["coordinates"],
			       circleOptions).addTo(map));
}


function drawBoth() {
    drawLinestrings();
    drawPolygon();
}


function resetData() {
    resetPolygonData();
    resetLinesData();    
}


function resetFigures() {
    resetPolygonFigure();
    resetLinesFigure();    
}


function resetAll() {
    resetFigures();
    resetData();
}


function resetPolygonData() {
    if (polygon !== undefined && polygon !== null) {
	polJson = null;
	polygon = null;
    }
}

function resetPolygonFigure() {
    resetMarker();
    if (polygon !== undefined && polygon !== null) {
	map.removeLayer(polygon)
    }    
}


function resetLinesData() {
    linJson = null;
    lines = [];
}


function resetLinesFigure() {
    resetMarker();
    lines.forEach((l) => map.removeLayer(l));
}

function resetMarker() {
    marker.forEach((m) => map.removeLayer(m));
    marker = [];
}


async function getPolygonResponse() {
    if (latlng === undefined || latlng === null) {
	return;
    }
    
    let polygon_response = await fetch('/polygon?lat=' +
				       latlng.lat +
				       '&lng=' +
				       latlng.lng +
				       '&cost=' +
				       cost);

    polJson = await polygon_response.json();
    
}


async function getLinestringResponse() {
    if (latlng === undefined || latlng === null) {
	return;
    }
    
    let multilinestring_response = await fetch('/multilinestring?lat=' +
					       latlng.lat +
					       '&lng=' +
					       latlng.lng +
					       '&cost=' +
					       cost);
    
    linJson = await multilinestring_response.json();
}
