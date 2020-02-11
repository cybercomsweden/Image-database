import React from "react";
import { NavigationControl } from "mapbox-gl";
import { BaseMap } from "./widgets/map.jsx";
import { Entities } from "./api.js";

export class WorldMap extends BaseMap {
    constructor(props) {
        super(props);
        this.state = {
            lng: 30,
            lat: 30,
            zoom: 1.6,
            entities: null,
        };
        this.registerMapRef = this.registerMapRef.bind(this);
    }

    registerMapRef(map) {
    // We utilize that BaseMap.mapRef is called before this.componentDidMount is
        this.map = map;
    }

    async getMetadata() {
        const entities = await Entities.fetch();
        const features = [];
        if (entities != null) {
            for (const entity of entities.entity) {
                const { location } = entity;
                if (location != null) {
                    features.push({
                        type: "Feature",
                        properties: {},
                        geometry: {
                            type: "Point",
                            coordinates: [location.longitude, location.latitude],
                        },

                    });
                }
            }
        }

        this.map.on("load", () => {
            this.map.loadImage("/static/mapbox-icon.png",
                (error, image) => {
                    if (error) throw error;
                    this.map.addImage("icon", image);
                    this.map.addSource("points", {
                        type: "geojson",
                        data: {
                            type: "FeatureCollection",
                            features,
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

    componentDidMount() {
        this.getMetadata();
        // Override positions from <Map />
        const { lng, lat, zoom } = this.state;
        this.map.setCenter([lng, lat]);
        this.map.setZoom(zoom);
    }

    render() {
        return (
            <div>
                <BaseMap mapRef={this.registerMapRef} className="mapContainer" />
                <div id="zoom" className="zoom">
                    <b><i>Zoom out</i></b>
                </div>
            </div>
        );
    }
}
