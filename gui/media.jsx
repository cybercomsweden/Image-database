import React from "react";
import { Link, Route, Switch } from "react-router-dom";
import { Entity, Entities } from "./api.js";
import { Map } from "./widgets/map.jsx";

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
        <svg className="chevron" width="50px" height="75px" viewBox="0 0 50 75">
            <polyline
                fill="none"
                stroke="#FFFFFF"
                strokeWidth="3"
                strokeLinecap="round"
                strokeLinejoin="round"
                points={points}
            />
        </svg>
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
                <Link className="button prev" to={`/media/${prevEntity.id}`}>
                    {createArrow("47,3 3,37.5 47,72")}
                </Link>
            );
        }
        let next = "";
        if (nextEntity != null) {
            next = (
                <Link className="button next" to={`/media/${nextEntity.id}`}>
                    {createArrow("3,3 47,37.5 1,72")}
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
                map = <Map className="preview-map" lng={location.longitude} lat={location.latitude} zoom="10" />;
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
                <div className="preview-metadata">
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

export class Media extends React.Component {
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
