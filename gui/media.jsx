import React from "react";
import {
    BrowserRouter, Link, NavLink, Route, Switch,
} from "react-router-dom";
import { Search } from "./search.jsx";
import { Entity, Entities, Tags } from "./api.js";
import { Map, WorldMap } from "./map.jsx";

function getFormattedDate(timestamp) {
    const date = new Date(timestamp * 1000);
    const year = date.getFullYear();
    const month = date.getMonth() + 1;
    const day = date.getDate();
    let hours = date.getHours();
    const minutes = `0${date.getMinutes()}`;
    const seconds = `0${date.getSeconds()}`;
    const timezoneOffset = date.getTimezoneOffset();
    if (timezoneOffset !== 0) {
        hours += timezoneOffset / 60;
    }
    return `${year}-${month}-${day} ${hours}:${minutes.substr(-2)}:${seconds.substr(-2)}`;
}

function createArrow(points) {
    return (
        <button className="arrow left" type="submit">
            <svg width="60px" height="80px" viewBox="0 0 50 80">
                <polyline
                    fill="none"
                    stroke="#FFFFFF"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    points={points}
                />
            </svg>
        </button>
    );
}

class Pic extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            entity: null,
        };
    }

    componentDidMount() {
        this.getEntity();
    }

    componentDidUpdate(previousProps) {
        const { entity } = this.props;
        if (previousProps.entity.id !== entity.id) {
            this.getEntity();
        }
    }

    async getEntity() {
        const { entity } = this.props;
        this.setState({ entity: await Entity.fetch(entity.id) });
    }

    render() {
        const { entity, prevEntity, nextEntity } = this.props;
        const { entity: entityMeta } = this.state;
        let prev = "";
        if (prevEntity != null) {
            prev = (
                <Link className="prev" to={`/media/${prevEntity.id}`}>
                    {createArrow("46.0,75.0 1.0,37.5 46.0,0.0")}
                </Link>
            );
        }
        let next = "";
        if (nextEntity != null) {
            next = (
                <Link className="next" to={`/media/${nextEntity.id}`}>
                    {createArrow("0.0,75.0 37.5,37.5 0.0,0.0")}
                </Link>
            );
        }
        let map = null;
        let metadata = "";
        let width; let height; let flash; let formattedDate = "Metadata not available";
        const name = entity.path.replace("dest/", "");
        if (entityMeta != null) {
            width = entityMeta.metadata.width;
            height = entityMeta.metadata.height;
            if (entityMeta.metadata.image != null) {
                if (entityMeta.created.seconds > 0) {
                    formattedDate = getFormattedDate(entityMeta.created.seconds);
                }
                if (entityMeta.metadata.image.flash) {
                    flash = "Yes";
                } else {
                    flash = "No";
                }
                metadata = (
                    <ul>
                        <li>
                            <strong>Filename: </strong>
                            {name}
                        </li>
                        <li>
                            <strong>Created: </strong>
                            {formattedDate}
                        </li>
                        <li>
                            <strong>Width: </strong>
                            {width}
                        </li>
                        <li>
                            <strong>Height: </strong>
                            {height}
                        </li>
                        <li>
                            <strong>Aperture: </strong>
                            {entityMeta.metadata.image.aperture.toFixed(1)}
                        </li>
                        <li>
                            <strong>ISO: </strong>
                            {entityMeta.metadata.image.iso}
                        </li>
                        <li>
                            <strong>Flash: </strong>
                            {flash}
                        </li>
                        <li>
                            <strong>Location: </strong>
                            {entityMeta.location.longitude.toFixed(1)}
                            ,
                            {entityMeta.location.latitude.toFixed(1)}
                            {entityMeta.location.place}
                        </li>
                    </ul>
                );
            } else if (entityMeta.metadata.type_specific === "video") {
                if (entityMeta.created.seconds > 0) {
                    formattedDate = getFormattedDate(entityMeta.created.seconds);
                }
                metadata = (
                    <ul>
                        <li>
                            <strong>Filename: </strong>
                            {name}
                        </li>
                        <li>
                            <strong>Created: </strong>
                            {formattedDate}
                        </li>
                        <li>
                            <strong>Width: </strong>
                            {width}
                        </li>
                        <li>
                            <strong>Height: </strong>
                            {height}
                        </li>
                        <li>
                            <strong>Duration: </strong>
                            {entityMeta.metadata.video.duration.toFixed(1)}
                            {" seconds"}
                        </li>
                        <li>
                            <strong>Location: </strong>
                            {entityMeta.location.longitude.toFixed(1)}
                            {", "}
                            {entityMeta.location.latitude.toFixed(1)}
                            {" "}
                            {entityMeta.location.place}
                        </li>
                    </ul>
                );
            } else {
                metadata = (
                    <ul>
                        <li>
                            <strong>Filename:</strong>
                            {name}
                        </li>
                    </ul>
                );
            }
            const { location } = entityMeta;
            if (location && (location.latitude || location.longitude)) {
                map = <Map lng={location.longitude} lat={location.latitude} zoom="10" />;
            }
        }


        return (
            <div className="preview-container">
                <div className="preview-media">
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
                <div className="preview-meta">
                    {metadata}
                    {map}
                </div>
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
                    <Route path="/map"><WorldMap /></Route>
                    <Route path="/"><Media /></Route>
                </Switch>
            </div>
        </BrowserRouter>
    );
}

export default App;
