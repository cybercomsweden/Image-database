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

function PlayButton() {
    return (
        <svg className="video-overlay-play-button" width="75px" height="75" viewBox="0 0 213.7 213.7" enableBackground="new 0 0 213.7 213.7">
            <polygon className="triangle" id="XMLID_18_" fill="none" strokeWidth="7" strokeLinecap="round" strokeLinejoin="round" strokeMiterlimit="10" points="73.5,62.5 148.5,105.8 73.5,149.1 " />
            <circle className="circle" id="XMLID_17_" fill="none" strokeWidth="7" strokeLinecap="round" strokeLinejoin="round" strokeMiterlimit="10" cx="106.8" cy="106.8" r="103.3" />
        </svg>

    );
}

function Metadata(props) {
    /* eslint-disable camelcase */
    const {
        data: metadata,
        filename,
        locationName,
        created,
        uploaded,
        ...attrs
    } = props;
    const { width, height, type_specific } = metadata;

    const items = [];

    if (filename != null) {
        items.push(<dt key="filename_key">Filename</dt>);
        items.push(<dd key="filename_value">{filename}</dd>);
    }

    if (created != null) {
        items.push(<dt key="created_key">Taken</dt>);
        items.push(<dd key="created_value">{getFormattedDate(created.seconds)}</dd>);
    }

    if (uploaded != null) {
        items.push(<dt key="uploaded_key">Uploaded</dt>);
        items.push(<dd key="uploaded_value">{getFormattedDate(uploaded.seconds)}</dd>);
    }

    if (width != null) {
        items.push(<dt key="width_key">Width</dt>);
        items.push(
            <dd key="width_value">
                {width}
                {" px"}
            </dd>,
        );
    }

    if (height != null) {
        items.push(<dt key="height_key">Height</dt>);
        items.push(
            <dd key="height_value">
                {height}
                {" px"}
            </dd>,
        );
    }

    if (locationName != null) {
        items.push(<dt key="location_name_key">Location</dt>);
        items.push(<dd key="location_name_value">{locationName}</dd>);
    }

    switch (type_specific) {
    case "image": {
        const {
            image: {
                exposure_time, aperture, iso, flash,
            },
        } = metadata;

        if (exposure_time != null && exposure_time !== 0) {
            items.push(<dt key="exposure_time_key">Exposure time</dt>);
            items.push(<dd key="exposure_time_value">{exposure_time}</dd>);
        }

        if (iso != null && iso !== 0) {
            items.push(<dt key="iso_key">ISO</dt>);
            items.push(<dd key="iso_value">{iso}</dd>);
        }

        if (aperture != null && aperture !== 0) {
            items.push(<dt key="aperture_key">Aperture</dt>);
            items.push(<dd key="aperture_value">{aperture.toFixed(1)}</dd>);
        }

        if (flash != null) {
            items.push(<dt key="flash_key">Flash</dt>);
            items.push(<dd key="flash_value">{flash ? "Yes" : "No"}</dd>);
        }
        break;
    }
    case "video": {
        const { video: { duration, rotation, frame_rate } } = metadata;

        if (duration != null) {
            items.push(<dt key="duration_key">Duration</dt>);
            items.push(
                <dd key="duration_value">
                    {duration.toFixed(1)}
                    {" seconds"}
                </dd>,
            );
        }

        // TODO: This should probably be shared between video and image
        if (rotation != null && rotation !== 0) {
            items.push(<dt key="rotation_key">Rotation</dt>);
            items.push(<dd key="rotation_value">{rotation}</dd>);
        }

        if (frame_rate != null && frame_rate !== 0) {
            items.push(<dt key="frame_rate_key">Frame rate</dt>);
            items.push(<dd key="frame_rate_value">{frame_rate}</dd>);
        }
        break;
    }
    default:
        throw new Error("Unexpected metadata type");
    }

    return <dl className="property-table" {...attrs}>{items}</dl>;
    /* eslint-enable camelcase */
}

function MapLogo({ width, height }) {
    return (
        <svg version="1.1" width={width} height={height} viewBox="0 0 1000 1000" enableBackground="new 0 0 1000 1000">
            <g>
                <path d="M698.7,554.7h114c17.1,0,36.6,12.3,44,27.8l0,0L637,621.3C657.8,598,678.7,576.3,698.7,554.7L698.7,554.7z M310,554.7H190.7l0,0L383.4,666l22.7-4C374.7,622.3,341.1,588.4,310,554.7z M541,757l245.5,141.7H969c17.2,0,25.5-12.7,18.4-28.4L876.7,626.6l0,0L591.1,677C573.1,701.1,556.1,727.4,541,757z M496.6,803.6l164.8,95.1l0,0H31c-17.3,0-25.6-12.7-18.4-28.4l45.1-99.2l0,0l293.1-51.7l138.7,80.1C491.6,801.3,494,802.7,496.6,803.6z M81.2,719.3l56-123.2l150.3,86.8L81.2,719.3L81.2,719.3z" />
                <path d="M514.8,698.7c-5.5,13.5-24.5,13.4-30-0.1C423.7,549,300.6,508,277.1,374.7c-23.2-131.2,67.8-259.8,200.7-272.3c135.1-12.8,248.8,93,248.8,225.3C726.6,500.4,582.6,532.7,514.8,698.7z M619.6,327.6c0-66-53.6-119.5-119.7-119.5c-66.1,0-119.7,53.5-119.7,119.5c0,66,53.6,119.5,119.7,119.5C566,447.2,619.6,393.7,619.6,327.6L619.6,327.6z" />
            </g>
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
        const { entity: simpleEntity, prevEntity, nextEntity } = this.props;
        const { entity: fullEntity } = this.state;

        let additionalInfo = null;
        if (fullEntity !== null) {
            const {
                uploaded, created, location, metadata,
            } = fullEntity;

            let map;
            if (location && (location.latitude || location.longitude)) {
                map = <Map className="preview-map" lng={location.longitude} lat={location.latitude} zoom="10" />;
            } else {
                map = (
                    <div className="preview-map no-map">
                        <MapLogo width="150" height="150" />
                        <span>No GPS data available</span>
                    </div>
                );
            }

            additionalInfo = (
                <div className="preview-metadata">
                    <Metadata
                        data={metadata}
                        filename={simpleEntity.path.replace("dest/", "")}
                        locationName={location ? location.place : null}
                        created={created}
                        uploaded={uploaded}
                    />
                    {map}
                </div>
            );
        }

        let overlay = null;
        if (simpleEntity.media_type === Entity.EntityType.VIDEO.value) {
          overlay = <PlayButton/>;
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
                    {
                        prevEntity != null && (
                            <Link className="button prev" to={`/media/${prevEntity.id}`}>
                                <Chevron dir="left" />
                            </Link>
                        )
                    }
                    <img className="preview" src={`/assets/${simpleEntity.preview_path}`} alt="" />
                    {overlay}
                    {
                        nextEntity != null && (
                            <Link className="button next" to={`/media/${nextEntity.id}`}>
                                <Chevron dir="right" />
                            </Link>
                        )
                    }
                </div>
                {additionalInfo}
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
              overlay = <PlayButton/>;
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
