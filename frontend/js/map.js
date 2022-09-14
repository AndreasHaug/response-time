// var map = L.map('map').setView([59.94, 10.75], 13.0);
var map = L.map('map').setView([67.283333, 14.383333], 13.0);



// var map = L.map('map').setView([69.64527778, 18.99277778], 13.0); 
      // L.tileLayer('https://api.maptiler.com/maps/streets/{z}/{x}/{y}.png?key=X8lQYMMJ147bLoWMQqfE').addTo(map);
      L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png').addTo(map);
      
let lines = [];
let polygon;
      let marker;
      map.on("click", (e) => {
	  let latlng = e.latlng;
	  console.log(latlng)
	  async function doSearch() {

	      let polygon_response = await fetch('/polygon?lat=' +
				       latlng.lat +
				       '&lon=' +
				       latlng.lng +
				      '&cost=2');

	      
	      // let multilinestring_response = await fetch('/multilinestring?lat=' +
	      // 			       latlng.lat +
	      // 			       '&lon=' +
	      // 			       latlng.lng +
	      // 			      '&cost=10');

	      
	      lines.forEach((l) => map.removeLayer(l))
	      if (marker !== undefined) {
		  map.removeLayer(marker); 		  
	      }
	      if (polygon !== undefined) {
		  map.removeLayer(polygon)
	      }

	      
	      let poljson = await polygon_response.json();
	      // let linjson = await multilinestring_response.json();

	      let circleOptions = {
		  radius: 4,
		  color: 'blue',
		  fillColor: 'blue',
		  fillOpacity: 1
	      }
	      
	      // for (let a in linjson["results"]["MultilinestringResult"]["coordinates"]) {
		  // lines.push(L.polyline(linjson["results"]["MultilinestringResult"]["coordinates"][a], {color: 'red'}).addTo(map));
	      // }
	      polygon = L.polygon(poljson["results"]["PolygonResult"]["coordinates"], {color: 'blue'}).addTo(map);
	      marker = L.circleMarker(poljson["start"]["coordinates"].reverse(), circleOptions).addTo(map);


	  }
	  doSearch();
      }
      );
