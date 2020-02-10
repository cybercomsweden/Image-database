import React from "react";
import mapboxgl from "mapbox-gl";

mapboxgl.accessToken = "pk.eyJ1IjoiYmFja2xvZyIsImEiOiJjazY3dWd5aTAxdWE3M2xxd251a2czeGFkIn0.8OLm6vH4B5aNnbIWnbYCUw";

export class Map extends React.Component {
    componentDidMount() {
        // TODO: Delete on umount
        const { lng, lat, zoom } = this.props;
        this.map = new mapboxgl.Map({
            container: this.container,
            style: "mapbox://styles/mapbox/streets-v11",
            center: [lng, lat],
            zoom,
        });
    }

    render() {
        const style = {
            height: "400px",
        };
        return <div ref={(el) => { this.container = el; }} style={style} />;
    }
}

export class WorldMap extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            lng: 30,
            lat: 30,
            zoom: 1.6,
        };
    }

    componentDidMount() {
        const { lng, lat, zoom } = this.state;
        const map = new mapboxgl.Map({
            container: this.mapContainer,
            style: "mapbox://styles/mapbox/streets-v11",
            center: [lng, lat],
            zoom,
        });

        map.on("load", () => {
            map.loadImage("/static/mapbox-icon.png",
                (error, image) => {
                    if (error) throw error;
                    map.addImage("icon", image);
                    map.addSource("points", {
                        type: "geojson",
                        data: {
                            type: "FeatureCollection",
                            features: [
                                {
                                    type: "Feature",
                                    properties: {},
                                    geometry: {
                                        type: "Point",
                                        coordinates: [
                                            -91.395263671875,
                                            -0.9145729757782163,
                                        ],
                                    },
                                },
                                {
                                    type: "Feature",
                                    properties: {},
                                    geometry: {
                                        type: "Point",
                                        coordinates: [
                                            -90.32958984375,
                                            -0.6344474832838974,
                                        ],
                                    },
                                },
                                {
                                    type: "Feature",
                                    properties: {},
                                    geometry: {
                                        type: "Point",
                                        coordinates: [
                                            15.621355,
                                            58.410869,
                                        ],
                                    },
                                },
                                {
                                    type: "Feature",
                                    properties: {},
                                    geometry: {
                                        type: "Point",
                                        coordinates: [
                                            -91.34033203125,
                                            0.01647949196029245,
                                        ],
                                    },
                                },
                                {
                                    type: "Feature",
                                    properties: {},
                                    geometry: {
                                        type: "Point",
                                        coordinates: [
                                            114.170883,
                                            22.312940,
                                        ],
                                    },
                                },
                                {
                                    type: "Feature",
                                    properties: {},
                                    geometry: {
                                        type: "Point",
                                        coordinates: [
                                            -74.004846,
                                            40.710842,
                                        ],
                                    },
                                },


                            ],
                        },
                    });
                    // Add a symbol layer.
                    map.addLayer({
                        id: "symbols",
                        type: "symbol",
                        source: "points",
                        layout: {
                            "icon-image": "icon",
                            "icon-size": 0.15,
                        },
                    });
                });
        });

        // Center the map on the coordinates of any clicked symbol from the 'symbols' layer.
        map.on("click", "symbols", (e) => {
            map.flyTo({ center: e.features[0].geometry.coordinates, zoom: 10 });
        });

        // zoom in and out
        map.addControl(new mapboxgl.NavigationControl({ showCompass: false }));

        // Change the cursor to a pointer when the it enters a feature in the 'symbols' layer.
        map.on("mouseenter", "symbols", () => {
            map.getCanvas().style.cursor = "pointer";
        });

        // Change it back to a pointer when it leaves.
        map.on("mouseleave", "symbols", () => {
            map.getCanvas().style.cursor = "";
        });

        document.getElementById("zoom").addEventListener("click", () => {
            map.setZoom(1.6);
            map.setCenter([30, 30]);
        });
    }

    render() {
        return (
            <div>
                <div ref={(el) => { this.mapContainer = el; }} className="mapContainer" />
                <div id="zoom" className="zoom">
                    <b><i>Zoom out</i></b>
                </div>
            </div>
        );
    }
}
