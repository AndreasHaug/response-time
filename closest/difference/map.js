var map = L.map('map').setView([59.94, 10.75], 13.0);
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png').addTo(map);

let closestOnLinestringMarker;
let closestPointMarker;

map.on("click", (e) => {
    let latlng = e.latlng;
    // console.log(latlng)
    async function doSearch() {

	let closestOnClosestLinestring = await fetch("/?lat=" + latlng.lat + "&lng=" + latlng.lng)
	let closestPoint = await fetch("/closest_point?lat=" + latlng.lat + "&lng=" + latlng.lng)
	
	let closestlsjson = await closestOnClosestLinestring.json();
	let closestpjson = await closestPoint.json();
	
	if (closestOnLinestringMarker !== undefined) {
	    map.removeLayer(closestOnLinestringMarker); 		  
	}
	if (closestPointMarker !== undefined) {
	    map.removeLayer(closestPointMarker); 		  
	}
	
	let closestOnClosestLinestringCoords = closestlsjson["geometry"]["coordinates"].reverse();
	let closestPointCoords = closestpjson["geometry"]["coordinates"].reverse();
	
	let circleOptions = {
	    radius: 4,
	    color: 'blue',
	    fillColor: 'blue',
	    fillOpacity: 1
	}

	if (closestOnClosestLinestringCoords[0] === closestPointCoords[0] &&
	    closestOnClosestLinestringCoords[1] === closestPointCoords[1]) {
	    closestPointMarker = L.circleMarker(closestPointCoords, {
		radius: 4,
		color: 'purple',
		fillColor: 'purple',
		fillOpacity: 1
	    }).addTo(map);
	}
	else {
	    closestPointMarker = L.circleMarker(closestPointCoords, {
		radius: 4,
		color: 'red',
		fillColor: 'red',
		fillOpacity: 1
	    }).addTo(map);

	    closestOnLinestringMarker = L.circleMarker(closestOnClosestLinestringCoords, {
		radius: 4,
		color: 'green',
		fillColor: 'green',
		fillOpacity: 1
	    }).addTo(map);
	}
    }
    doSearch();
});
