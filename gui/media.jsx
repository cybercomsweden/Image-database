import React from "react";
import mapboxgl from "mapbox-gl";
import {
    BrowserRouter, Link, NavLink, Route, Switch,
} from "react-router-dom";
import { Search } from "./search.jsx";
import { Entities, Tags } from "./api.js";

mapboxgl.accessToken = "pk.eyJ1IjoiYmFja2xvZyIsImEiOiJjazY3dWd5aTAxdWE3M2xxd251a2czeGFkIn0.8OLm6vH4B5aNnbIWnbYCUw";
// Temporarily disable warning since component will have state later
// eslint-disable-next-line react/prefer-stateless-function
class Pic extends React.Component {
    render() {
        const { entity, prevEntity, nextEntity } = this.props;
        let prev = "";
        if (prevEntity != null) {
            prev = (
                <Link className="prev" to={`/media/${prevEntity.id}`}>
                    <button className="arrow left" type="submit">
                        <svg width="60px" height="80px" viewBox="0 0 50 80">
                            <polyline
                                fill="none"
                                stroke="#FFFFFF"
                                strokeWidth="2"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                points="46.0,75.0 1.0,37.5 46.0,0.0"
                            />
                        </svg>
                    </button>
                </Link>
            );
        }
        let next = "";
        if (nextEntity != null) {
            next = (
                <Link className="next" to={`/media/${nextEntity.id}`}>
                    <button className="arrow right" type="submit">
                        <svg width="60px" height="80px" viewBox="0 0 50 80">
                            <polyline
                                fill="none"
                                stroke="#FFFFFF"
                                strokeWidth="2"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                points="0.0,75.0 37.5,37.5 0.0,0.0"
                            />
                        </svg>
                    </button>
                </Link>
            );
        }
        return (
            <div className="preview_div">
                <Link className="close" to="/">
                    <svg width="20px" height="20px">
                        <line x1="2" y1="2" x2="20" y2="20" stroke="white" strokeWidth="2" />
                        <line x1="20" y1="2" x2="2" y2="20" stroke="white" strokeWidth="2" />
                    </svg>
                </Link>
                {prev}
                <img className="preview" src={`/assets/${entity.preview_path}`} alt="" />
                {next}
            </div>
        );
    }
}

function MediaList({ entities }) {
    const entityLinks = [];
    if (entities != null) {
        for (const entity of entities) {
            entityLinks.push(
                <Link className="media-thumbnail" key={entity.id} to={`/media/${entity.id}`}>
                    <img src={`/assets/${entity.thumbnail_path}`} alt="" />
                </Link>,
            );
        }
    }
    return (
        <div className="media-thumbnail-list">
            {entityLinks}
        </div>
    );
}

class Media extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            entities: null,
        };
    }

    componentDidMount() {
        this.getThumbnails();
    }

    async getThumbnails() {
        this.setState({ entities: await Entities.fetch() });
    }

    render() {
        const { entities: entitiesPb } = this.state;
        if (entitiesPb === null) {
            return "Loading";
        }
        const entities = entitiesPb.entity;
        return (
            <Switch>
                <Route exact path="/"><MediaList entities={entities} /></Route>
                <Route
                    exact
                    path="/media/:id"
                    children={({ match }) => {
                        for (let i = 0; i < entities.length; i += 1) {
                            const entity = entities[i];
                            if (entity.id !== parseInt(match.params.id, 10)) {
                                continue;
                            }
                            let prevEntity = null;
                            if (i > 0) {
                                prevEntity = entities[i - 1];
                            }
                            let nextEntity = null;
                            if (i < entities.length - 1) {
                                nextEntity = entities[i + 1];
                            }
                            return (
                                <Pic
                                    entity={entity}
                                    prevEntity={prevEntity}
                                    nextEntity={nextEntity}
                                />
                            );
                        }
                        return "No image found";
                    }}
                />
            </Switch>
        );
    }
}

class ApiTags extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            tags: null,
        };
    }

    componentDidMount() {
        this.getTags();
    }

    async getTags() {
        this.setState({ tags: await Tags.fetch() });
    }

    render() {
        const { tags: tagsPb } = this.state;
        if (tagsPb === null) {
            return "Loading";
        }
        if (!tagsPb.tag.length) {
            return "No tags yet";
        }
        const tags = [];
        for (const tag of tagsPb.tag) {
            tags.push(<div key={tag.id}>{tag.canonical_name}</div>);
        }
        return (
            <div className="tag-list">
                {tags}
            </div>
        );
    }
}

class WorldMap extends React.Component {
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

function Map() {
    return (
        <WorldMap />
    );
}

function App() {
    return (
        <BrowserRouter>
            <div className="content">
                <header>
                    <nav>
                        <NavLink to="/" isActive={(match, location) => location.pathname === "/" || location.pathname.match(/^\/media\//) !== null}>Media</NavLink>
                        <NavLink to="/tags">Tags</NavLink>
                        <NavLink to="/map">Map</NavLink>
                    </nav>
                    <Search />
                </header>
                <Switch>
                    <Route path="/tags"><ApiTags /></Route>
                    <Route path="/map"><Map /></Route>
                    <Route path="/"><Media /></Route>
                </Switch>
            </div>
        </BrowserRouter>
    );
}

export default App;
