import update from "immutability-helper";
import React from "react";
import {
    Link, Route, Switch, withRouter,
} from "react-router-dom";
import queryString from "query-string";
import { Entity, Entities } from "./api.js";
import { Map } from "./widgets/map.jsx";
import { SimpleSearch } from "./widgets/search.jsx";

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

/**
 * Just like <Link /> but preserves query string parameters
 */
const PreserveQueryParamsLink = withRouter((props) => {
    const {
        match, location, history, staticContext, to, children, ...attrs
    } = props;
    return <Link to={`${to}${location.search}`} {...attrs}>{children}</Link>;
});

function EditIcon(props) {
    return (
        <svg version="1.1" viewBox="0 0 1000 1000" {...props}>
            <g>
                <path d="M158.1,990.4c-8.8-1.5-17.8-2.5-26.4-4.7C74.9,971.1,37,935.7,17.6,880.6C11.8,864,10,846.7,10,829.2c0-209,0-418,0.1-627c0-55.7,22.7-100.2,68-132.7c24.4-17.5,52.1-26.4,82.1-28c4.1-0.2,8.2-0.3,12.3-0.3c164.4,0,328.8,0,493.2,0c2.1,0,4.2,0,7.3,0c-1.5,1.8-2.3,3-3.3,4c-29,29.1-58,58.1-87.2,87c-2.2,2.1-6.1,3.5-9.1,3.5c-123.4,0.2-246.7,0.3-370.1,0.1c-29.9,0-55,10-75,32.3c-16.5,18.5-23.9,40.5-23.9,65c0,188.3-0.1,376.5,0,564.8c0,47.7,29.9,85.2,75.5,95.2c7.4,1.6,15.1,2.1,22.7,2.1c187.9,0.1,375.7,0.1,563.6,0.1c56.7,0,98-41.4,98-98.1c0.1-123.4,0-246.7-0.1-370.1c0-4.7,1.4-7.9,4.7-11.2c28.3-28.1,56.5-56.3,84.6-84.6c1.4-1.4,2.2-3.2,3.3-4.9c0.6,0.4,1.1,0.7,1.7,1.1c0.1,1.6,0.4,3.3,0.4,4.9c0,165.4,0.2,330.8-0.1,496.2c-0.1,55.5-22.5,100.2-67.8,132.8c-22.1,16-47.2,24.8-74.4,27.6c-2.1,0.2-4.2,0.7-6.3,1.1C592.9,990.4,375.5,990.4,158.1,990.4z" />
                <path d="M858.5,9.6c14.9,1.5,24.9,10.8,34.8,20.8c27.4,27.6,55.1,54.9,82.7,82.3c17.7,17.6,19,40.8,2.1,59.1c-14.8,16.1-30.8,31-46.8,46.9C881.5,168,831.8,117.5,782.2,66.9c-0.7,0.4-1.3,0.9-1.9,1.3c1.1-1.3,2.2-2.6,3.4-3.8c10.9-10.9,22.2-21.5,32.8-32.7c10.4-11,21.8-20,37.4-22.1C855.3,9.6,856.9,9.6,858.5,9.6z" />
                <path d="M425.1,420.8C534.2,312.4,645,202.2,756.3,91.7c49.8,50.7,99.4,101.3,149.6,152.5c-109,108.3-219.9,218.5-331.1,329C525,522.4,475.3,471.9,425.1,420.8z" />
                <path d="M401.8,443.7c50.4,51.2,99.6,101.4,149.1,151.7c-68.8,19.5-138.8,39.4-209.8,59.6C361.3,584.5,381.3,514.9,401.8,443.7z" />
            </g>
        </svg>
    );
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

class PlayButton extends React.Component {
    constructor(props) {
        super(props);
        this.handleClick = this.handleClick.bind(this);
    }

    handleClick() {
        const { onClick } = this.props;
        onClick();
    }

    render() {
        return (
            <svg onClick={this.handleClick} className="video-overlay-play-button" width="75px" height="75" viewBox="0 0 213.7 213.7" enableBackground="new 0 0 213.7 213.7">
                <polygon className="triangle" id="XMLID_18_" fill="none" strokeWidth="7" strokeLinecap="round" strokeLinejoin="round" strokeMiterlimit="10" points="73.5,62.5 148.5,105.8 73.5,149.1 " />
                <circle className="circle" id="XMLID_17_" fill="none" strokeWidth="7" strokeLinecap="round" strokeLinejoin="round" strokeMiterlimit="10" cx="106.8" cy="106.8" r="103.3" />
            </svg>
        );
    }
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
    const {
        width, height, rotation, type_specific,
    } = metadata;

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

    if (width != null && width !== 0) {
        items.push(<dt key="width_key">Width</dt>);
        items.push(
            <dd key="width_value">
                {width}
                {" px"}
            </dd>,
        );
    }

    if (height != null && height !== 0) {
        items.push(<dt key="height_key">Height</dt>);
        items.push(
            <dd key="height_value">
                {height}
                {" px"}
            </dd>,
        );
    }

    if (rotation != null && rotation !== 0) {
        let rotation_value;
        switch (rotation) {
        case 1: {
            rotation_value = "Cw90";
            break;
        }
        case 2: {
            rotation_value = "Ccw90";
            break;
        }
        case 3: {
            rotation_value = "Cw180";
            break;
        }
        default:
            // Unknown rotation
        }
        items.push(<dt key="rotation_key">Rotation</dt>);
        items.push(<dd key="rotation_value">{rotation_value}</dd>);
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
        const { video: { duration, frame_rate } } = metadata;

        if (duration != null) {
            items.push(<dt key="duration_key">Duration</dt>);
            items.push(
                <dd key="duration_value">
                    {duration.toFixed(1)}
                    {" seconds"}
                </dd>,
            );
        }

        if (frame_rate != null && frame_rate !== 0) {
            items.push(<dt key="frame_rate_key">Frame rate</dt>);
            items.push(<dd key="frame_rate_value">{frame_rate}</dd>);
        }
        break;
    }
    default:
        // This happens when the object is neither image nor video
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

function VideoPlayer(props) {
    const { entity } = props;
    return (
        // eslint-disable-next-line jsx-a11y/media-has-caption
        <video
            className="preview"
            src={`/assets/${entity.path}`}
            controls
            autoPlay
        />
    );
}

class Pic extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            entity: null,
            playClicked: false,
            edit: false,
        };

        this.tags = React.createRef();

        this.handlePlayClick = this.handlePlayClick.bind(this);
        this.handleEditClick = this.handleEditClick.bind(this);
        this.handleTagAdd = this.handleTagAdd.bind(this);
        this.handleTagRemove = this.handleTagRemove.bind(this);
    }


    componentDidMount() {
        this.getEntity();
    }

    componentDidUpdate(previousProps) {
        const { entity } = this.props;
        if (previousProps.entity.id !== entity.id) {
            // eslint-disable-next-line react/no-did-update-set-state
            this.setState({ playClicked: false });
            this.getEntity();
        }
    }

    async getEntity() {
        const { entity } = this.props;
        this.setState({ entity: await Entity.fetch(entity.id) });
    }

    handlePlayClick() {
        this.setState({ playClicked: true });
    }

    handleEditClick() {
        const { edit } = this.state;
        this.setState({ edit: !edit });
        if (!edit) {
            window.scrollTo({
                top: this.tags.offsetTop,
                behavior: "smooth",
            });
        }
    }

    async handleTagAdd(tag) {
        const { entity } = this.state;
        const newEntity = update(entity, {
            tags: { tag: { $push: [tag] } },
        });
        this.setState({ entity: await Entity.save(newEntity) });
    }

    async handleTagRemove(event) {
        const { entity } = this.state;
        const newTags = entity.tags.tag.filter(
            (tag) => tag.canonical_name !== event.currentTarget.dataset.canonicalName,
        );
        const newEntity = update(entity, {
            tags: { tag: { $set: newTags } },
        });
        this.setState({ entity: await Entity.save(newEntity) });
    }

    render() {
        const { entity: simpleEntity, prevEntity, nextEntity } = this.props;
        const { entity: fullEntity, edit } = this.state;

        let additionalInfo = null;
        const tags = [];
        if (fullEntity !== null) {
            const {
                uploaded, created, location, metadata, tags: tagList,
            } = fullEntity;

            if (tagList.tag.length) {
                for (const tag of tagList.tag) {
                    if (edit) {
                        tags.push(
                            <span className="tag" key={tag.canonical_name}>
                                {tag.name}
                                {" "}
                                <svg className="inherit-color clickable" width="12px" height="12px" onClick={this.handleTagRemove} data-canonical-name={tag.canonical_name}>
                                    <line x1="1" y1="1" x2="11" y2="11" strokeWidth="2" />
                                    <line x1="11" y1="1" x2="1" y2="11" strokeWidth="2" />
                                </svg>
                            </span>,
                        );
                    } else {
                        const dest = "/media?q=".concat(tag.canonical_name);
                        tags.push(
                            <Link className="tag" key={tag.canonical_name} to={dest}>
                                {tag.name}
                            </Link>,
                        );
                    }
                }
            }
            if (edit) {
                tags.push(
                    <div className="preview-edit" key="!search-input">
                        <SimpleSearch placeholder="Add tag" onSelect={this.handleTagAdd} />
                    </div>,
                );
            }

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
        const { playClicked } = this.state;
        if (simpleEntity.media_type === Entity.EntityType.VIDEO.value && !playClicked) {
            overlay = <PlayButton onClick={this.handlePlayClick} entity={simpleEntity} />;
        }

        let previewImg = null;
        let videoPlayer = null;
        if (!playClicked) {
            previewImg = <img className="preview" src={`/assets/${simpleEntity.preview_path}`} alt="" />;
        }
        if (playClicked) {
            videoPlayer = <VideoPlayer entity={simpleEntity} />;
        }

        return (
            <div className="preview-container">
                <div className="preview-media">
                    <div className="nav-bar">
                        <EditIcon className="inherit-color" width="20" height="20" onClick={this.handleEditClick} />
                        <PreserveQueryParamsLink className="button close" to="/">
                            <svg className="inherit-color" width="20px" height="20px">
                                <line x1="2" y1="2" x2="20" y2="20" strokeWidth="2" />
                                <line x1="20" y1="2" x2="2" y2="20" strokeWidth="2" />
                            </svg>
                        </PreserveQueryParamsLink>
                    </div>
                    {
                        prevEntity != null && (
                            <PreserveQueryParamsLink className="button prev" to={`/media/${prevEntity.id}`}>
                                <Chevron dir="left" />
                            </PreserveQueryParamsLink>
                        )
                    }
                    {previewImg}
                    {videoPlayer}
                    {overlay}
                    {
                        nextEntity != null && (
                            <PreserveQueryParamsLink className="button next" to={`/media/${nextEntity.id}`}>
                                <Chevron dir="right" />
                            </PreserveQueryParamsLink>
                        )
                    }
                </div>
                <div className="preview-tags" ref={(ref) => { this.tags = ref; }}>{tags}</div>
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
                overlay = <PlayButton />;
            }
            entityLinks.push(
                <PreserveQueryParamsLink className="media-thumbnail" key={entity.id} to={`/media/${entity.id}`}>
                    <img src={`/assets/${entity.thumbnail_path}`} alt="" />
                    {overlay}
                </PreserveQueryParamsLink>,
            );
        }
    }
    return (
        <div className="media-thumbnail-list">
            {entityLinks}
        </div>
    );
}

class InnerMedia extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            query: "",
            entities: null,
        };
    }

    componentDidMount() {
        const { location } = this.props;
        const params = queryString.parse(location.search);

        this.setState(
            { query: params.q },
            () => this.getThumbnails(params.q),
        );
    }

    componentDidUpdate() {
        const { location } = this.props;
        const { query } = this.state;
        const params = queryString.parse(location.search);

        if (params.q !== query) {
            // eslint-disable-next-line react/no-did-update-set-state
            this.setState(
                { query: params.q },
                () => this.getThumbnails(params.q),
            );
        }
    }

    async getThumbnails(query) {
        this.setState({ entities: await Entities.fetch(query) });
    }

    render() {
        const { entities: entitiesPb } = this.state;
        if (entitiesPb === null) {
            return "Loading";
        }
        const entities = entitiesPb.entity;
        return (
            <Switch>
                <Route exact path={["/", "/media"]} render={() => <MediaList entities={entities} />} />
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

export const Media = withRouter(InnerMedia);
