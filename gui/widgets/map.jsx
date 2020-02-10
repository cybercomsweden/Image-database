import React from "react";
import mapboxgl from "mapbox-gl";

// Ensure that Mapbox CSS gets bundled by Parcel
import "mapbox-gl/dist/mapbox-gl.css";


export class BaseMap extends React.Component {
    constructor(props) {
        super(props);
        this.map = React.createRef();
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
    }

    componentDidUpdate({ lng: prevLng, lat: prevLat, zoom: prevZoom }) {
        const { lng, lat, zoom } = this.props;
        if (prevLng !== lng || prevLat !== lat || prevZoom !== zoom) {
            this.map.setZoom(zoom);
            this.map.setCenter([lng, lat]);
        }
    }

    registerMapRef(map) {
        // We utilize that BaseMap.mapRef is called before this.componentDidMount is
        this.map = map;
    }

    render() {
        const style = {
            height: "400px",
        };
        return <BaseMap mapRef={this.registerMapRef} style={style} />;
    }
}
