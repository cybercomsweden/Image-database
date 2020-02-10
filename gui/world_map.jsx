import React from "react";
import { NavigationControl } from "mapbox-gl";
import { BaseMap } from "./widgets/map.jsx";

export class WorldMap extends BaseMap {
    constructor(props) {
        super(props);
        this.state = {
            lng: 30,
            lat: 30,
            zoom: 1.6,
        };
    }

    componentDidMount() {
        BaseMap.prototype.componentDidMount.call(this);

        // Override positions from <Map />
        const { lng, lat, zoom } = this.state;
        this.map.setCenter([lng, lat]);
        this.map.setZoom(zoom);

        this.map.on("load", () => {
            this.map.loadImage("/static/mapbox-icon.png",
                (error, image) => {
                    if (error) throw error;
                    this.map.addImage("icon", image);
                    this.map.addSource("points", {
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
                    this.map.addLayer({
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
        this.map.on("click", "symbols", (e) => {
            this.map.flyTo({ center: e.features[0].geometry.coordinates, zoom: 10 });
        });

        // zoom in and out
        this.map.addControl(new NavigationControl({ showCompass: false }));

        // Change the cursor to a pointer when the it enters a feature in the 'symbols' layer.
        this.map.on("mouseenter", "symbols", () => {
            this.map.getCanvas().style.cursor = "pointer";
        });

        // Change it back to a pointer when it leaves.
        this.map.on("mouseleave", "symbols", () => {
            this.map.getCanvas().style.cursor = "";
        });

        document.getElementById("zoom").addEventListener("click", () => {
            this.map.setZoom(1.6);
            this.map.setCenter([30, 30]);
        });
    }

    render() {
        return (
            <div>
                <div ref={(el) => { this.container = el; }} className="mapContainer" />
                <div id="zoom" className="zoom">
                    <b><i>Zoom out</i></b>
                </div>
            </div>
        );
    }
}
