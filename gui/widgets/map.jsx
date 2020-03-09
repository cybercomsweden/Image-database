Image database, allows the user to host a database themselves,
with the possibilities to tag and search after images.
Copyright (C) 2020 Cybercom group AB, Sweden
By Christoffer Dahl, Johanna Hultberg, Andreas Runfalk and Margareta Vi

Image database is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
import React from "react";
import mapboxgl from "mapbox-gl";

// Ensure that Mapbox CSS gets bundled by Parcel
import "mapbox-gl/dist/mapbox-gl.css";

export class BaseMap extends React.Component {
    constructor(props) {
        super(props);
        this.map = null;
        this.container = React.createRef();
    }

    componentDidMount() {
        this.map = new mapboxgl.Map({
            container: this.container,
            style: "mapbox://styles/mapbox/streets-v11",
            accessToken: "pk.eyJ1IjoiYmFja2xvZyIsImEiOiJjazY3dWd5aTAxdWE3M2xxd251a2czeGFkIn0.8OLm6vH4B5aNnbIWnbYCUw",
        });

        const { mapRef } = this.props;
        if (mapRef) {
            mapRef(this.map);
        }
    }

    componentDidUpdate() {
        // This is needed since the map won't use all available space if the container
        // is resized otherwise. This happens when switching between preview pages
        if (this.map !== null) {
            this.map.resize();
        }
    }

    componentWillUnmount() {
        if (this.map !== null) {
            if (this.mapRef) {
                this.mapRef(null);
            }
            this.map.remove();
            this.map = null;
        }
    }

    render() {
        const { mapRef, ...attrs } = this.props;
        return <div ref={(el) => { this.container = el; }} {...attrs} />;
    }
}

export class Map extends React.Component {
    constructor(props) {
        super(props);
        this.registerMapRef = this.registerMapRef.bind(this);
    }

    componentDidMount() {
        // NOTE: We must zoom before setCenter or the coordinates will be wrong
        const { lng, lat, zoom } = this.props;
        this.map.setZoom(zoom);
        this.map.setCenter([lng, lat]);
        this.marker = new mapboxgl.Marker()
            .setLngLat([lng, lat])
            .addTo(this.map);
    }

    componentDidUpdate({ lng: prevLng, lat: prevLat, zoom: prevZoom }) {
        const { lng, lat, zoom } = this.props;
        if (prevLng !== lng || prevLat !== lat || prevZoom !== zoom) {
            this.marker.setLngLat([lng, lat]);
            this.map.setZoom(zoom);
            this.map.setCenter([lng, lat]);
        }
    }

    registerMapRef(map) {
        // We utilize that BaseMap.mapRef is called before this.componentDidMount is
        this.map = map;
    }

    render() {
        return <BaseMap mapRef={this.registerMapRef} {...this.props} />;
    }
}
