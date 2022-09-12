var map = L.map('map').setView([59.94, 10.75], 13.0);
// var map = L.map('map').setView([67.283333, 14.383333], 13.0);



// var map = L.map('map').setView([69.64527778, 18.99277778], 13.0); 
      // L.tileLayer('https://api.maptiler.com/maps/streets/{z}/{x}/{y}.png?key=X8lQYMMJ147bLoWMQqfE').addTo(map);
      L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png').addTo(map);
      
// let lines = [];
// let polygon;
      let marker;
      map.on("click", (e) => {
	  let latlng = e.latlng;
	  // console.log(latlng)
	  async function doSearch() {

	      let resp = await fetch("/?lat=" + latlng.lat + "&lng=" + latlng.lng)
	      
	      // console.log(resp)
	      let respjson = await resp.json();

	      if (marker !== undefined) {
		  map.removeLayer(marker); 		  
	      }
	      
	      let circleLatLng = respjson["geometry"]["coordinates"].reverse()
	      console.log(circleLatLng)
	      // let circleLatLng = [linestringjson["start"]["lat"], linestringjson["start"]["lng"]];
	      // let circleLatLng = linestringjson["start"]["coordinates"]
	      // let circleLatLng = poljson["start"]["coordinates"]
	      
	      let circleOptions = {
		  radius: 4,
		  color: 'blue',
		  fillColor: 'blue',
		  fillOpacity: 1
	      }
	      
	      // for (let a in linestringjson["coordinates"]) {
		  // lines.push(L.polyline(linestringjson["coordinates"][a], {color: 'red'}).addTo(map));
	      // }
	      
	      // console.log(poljson["coordinates"])
	      // for (let a in s["coordinates"]) {
	      // polygon = L.polygon(poljson["coordinates"], {color: 'blue'}).addTo(map);
	      // }
	      
	      marker = L.circleMarker(circleLatLng, circleOptions).addTo(map);


	  }
	  doSearch();
      }
      );
