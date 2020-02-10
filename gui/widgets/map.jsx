import React from "react";
import mapboxgl from "mapbox-gl";

mapboxgl.accessToken = "pk.eyJ1IjoiYmFja2xvZyIsImEiOiJjazY3dWd5aTAxdWE3M2xxd251a2czeGFkIn0.8OLm6vH4B5aNnbIWnbYCUw";

export class BaseMap extends React.Component {
    componentDidMount() {
        if (!this.container) {
            throw new Error(
                "Unable to find this.container, you must define it in your render "
                + "function using ref={(el) => { this.container = el; }}",
            );
        }
        // TODO: Delete on umount
        this.map = new mapboxgl.Map({
            container: this.container,
            style: "mapbox://styles/mapbox/streets-v11",
        });
    }
}

export class Map extends React.Component {
    componentDidMount() {
        BaseMap.prototype.componentDidMount.call(this);

        // NOTE: We must zoom before setCenter or the coordinates will be wrong
        const { lng, lat, zoom } = this.props;
        this.map.setZoom(zoom);
        this.map.setCenter([lng, lat]);
    }

    render() {
        const style = {
            height: "400px",
        };
        return <div ref={(el) => { this.container = el; }} style={style} />;
    }
}
