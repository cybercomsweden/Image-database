import React from "react";
import {
    BrowserRouter, Link, NavLink, Route, Switch,
} from "react-router-dom";
import { Search } from "./search.jsx";
import { Entities, Tags } from "./api.js";

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

function App() {
    return (
        <BrowserRouter>
            <div className="content">
                <header>
                    <nav>
                        <NavLink to="/" isActive={(match, location) => location.pathname === "/" || location.pathname.match(/^\/media\//) !== null}>Media</NavLink>
                        <NavLink to="/tags">Tags</NavLink>
                    </nav>
                    <Search />
                </header>
                <Switch>
                    <Route path="/tags"><ApiTags /></Route>
                    <Route path="/"><Media /></Route>
                </Switch>
            </div>
        </BrowserRouter>
    );
}

export default App;
