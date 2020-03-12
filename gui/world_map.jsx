/*
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
*/
import React from "react";
import { NavigationControl, LngLatBounds } from "mapbox-gl";
import { BaseMap } from "./widgets/map.jsx";
import { Entities } from "./api.js";

import classes from "./css/world-map.css";

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

    async getFeaturePoints() {
        let { entities } = this;
        if (entities == null) {
            entities = await Entities.fetch();
        }
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
        return features;
    }

    fitMap(features) {
        const bounds = new LngLatBounds();
        features.forEach((feature) => {
            bounds.extend(feature.geometry.coordinates);
        });
        this.map.fitBounds(bounds, { padding: 100 });
    }

    async getMetadata() {
        const features = await this.getFeaturePoints();

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

        // zoom in and out controlls at top right corner of map
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
            this.fitMap(features);
        });
    }

    async componentDidMount() {
        this.getMetadata();
        // Override positions from <Map />
        const { lng, lat, zoom } = this.state;
        this.map.setCenter([lng, lat]);
        this.map.setZoom(zoom);

        const features = await this.getFeaturePoints();
        this.fitMap(features);
    }

    render() {
        return (
            <div>
                <BaseMap mapRef={this.registerMapRef} className={classes.container} />
                <div id="zoom" className={classes.zoom}>
                    <b><i>Zoom out</i></b>
                </div>
            </div>
        );
    }
}
