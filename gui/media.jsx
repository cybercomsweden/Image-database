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

function Chevron(props) {
    const { dir, ...other } = props;

    let points = null;
    if (dir === "left") {
        points = "47,3 3,37.5 47,72";
    } else if (dir === "right") {
        points = "3,3 47,37.5 1,72";
    } else {
        throw new Error("Cheron dir attribute must be either 'left' or 'right'");
    }

    return (
        <svg className="chevron" width="50px" height="75px" viewBox="0 0 50 75" {...other}>
            <polyline
                fill="none"
                stroke="#FFFFFF"
                strokeWidth="4"
                strokeLinecap="round"
                strokeLinejoin="round"
                points={points}
            />
        </svg>
    );
}

function createPlayButton() {
    return (
        <svg className="video-overlay-play-button" width="75px" height="75" viewBox="0 0 213.7 213.7" enableBackground="new 0 0 213.7 213.7">
            <polygon className="triangle" id="XMLID_18_" fill="none" strokeWidth="7" strokeLinecap="round" strokeLinejoin="round" strokeMiterlimit="10" points="73.5,62.5 148.5,105.8 73.5,149.1 " />
            <circle className="circle" id="XMLID_17_" fill="none" strokeWidth="7" strokeLinecap="round" strokeLinejoin="round" strokeMiterlimit="10" cx="106.8" cy="106.8" r="103.3" />
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
                    <Chevron dir="left" />
                </Link>
            );
        }
        let next = "";
        if (nextEntity != null) {
            next = (
                <Link className="button next" to={`/media/${nextEntity.id}`}>
                    <Chevron dir="right" />
                </Link>
            );
        }
        let map = null;
        let metadata = "";
        let width; let height; let flash; let formattedDate; let place;
        const name = entity.path.replace("dest/", "");
        let overlay = null;
        if (entityMeta != null) {
            width = entityMeta.metadata.width;
            height = entityMeta.metadata.height;
            if (entityMeta.location != null) {
                place = entityMeta.location.place;
            }
            if (entityMeta.metadata.image != null) {
                if (entityMeta.created !== null && entityMeta.created.seconds > 0) {
                    formattedDate = getFormattedDate(entityMeta.created.seconds);
                }
                if (entityMeta.metadata.image.flash) {
                    flash = "Yes";
                } else {
                    flash = "No";
                }
                metadata = (
                    <dl className="property-table">
                        <dt>Filename</dt>
                        <dd>{name}</dd>
                        <dt>Created</dt>
                        <dd>{formattedDate}</dd>
                        <dt>Width</dt>
                        <dd>{width}</dd>
                        <dt>Height</dt>
                        <dd>{height}</dd>
                        <dt>Aperture</dt>
                        <dd>{entityMeta.metadata.image.aperture.toFixed(1)}</dd>
                        <dt>ISO</dt>
                        <dd>{entityMeta.metadata.image.iso}</dd>
                        <dt>Flash</dt>
                        <dd>{flash}</dd>
                        <dt>Location</dt>
                        <dd>{place}</dd>
                    </dl>
                );
            } else if (entityMeta.metadata.type_specific === "video") {
                overlay = createPlayButton();
                if (entityMeta.created != null && entityMeta.created.seconds > 0) {
                    formattedDate = getFormattedDate(entityMeta.created.seconds);
                }
                metadata = (
                    <dl className="property-table">
                        <dt>Filename</dt>
                        <dd>{name}</dd>
                        <dt>Created</dt>
                        <dd>{formattedDate}</dd>
                        <dt>Width</dt>
                        <dd>{width}</dd>
                        <dt>Height</dt>
                        <dd>{height}</dd>
                        <dt>Duration</dt>
                        <dd>
                            {entityMeta.metadata.video.duration.toFixed(1)}
                            {" "}
                            seconds
                        </dd>
                        <dt>Location</dt>
                        <dd>{place}</dd>
                    </dl>
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
                    <Link className="button close" to="/">
                        <svg width="20px" height="20px">
                            <line x1="2" y1="2" x2="20" y2="20" stroke="white" strokeWidth="2" />
                            <line x1="20" y1="2" x2="2" y2="20" stroke="white" strokeWidth="2" />
                        </svg>
                    </Link>
                    {prev}
                    <img className="preview" src={`/assets/${entity.preview_path}`} alt="" />
                    {overlay}
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
            let overlay = null;
            if (entity.media_type === 1) {
                overlay = createPlayButton();
            }
            entityLinks.push(
                <Link className="media-thumbnail" key={entity.id} to={`/media/${entity.id}`}>
                    <img src={`/assets/${entity.thumbnail_path}`} alt="" />
                    {overlay}
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
