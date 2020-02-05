import Pbf from 'pbf';
import React from 'react';
import { BrowserRouter, Switch, Route, Link, useParams } from 'react-router-dom';
import Search from './search.js';
import {Entity, Entities} from './api.js';

class Pic extends React.Component {
    constructor(props) {
        super(props);
    }

    render() {
        let prev = ""
        if(this.props["prev-entity"] != null) {
            prev = <Link className="prev" to={`/media/${this.props["prev-entity"].id}`}>
                <button className="arrow left">
                <svg width="60px" height="80px" viewBox="0 0 50 80">
                    <polyline fill="none" stroke="#FFFFFF" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" points="
	                    46.0,75.0 1.0,37.5 46.0,0.0 "/>
                </svg>
                </button>
                </Link>;
        }
        let next = ""
        if(this.props["next-entity"] != null) {
            next = <Link className="next" to={`/media/${this.props["next-entity"].id}`}>
                                <button className="arrow right">
                <svg width="60px" height="80px" viewBox="0 0 50 80">
                    <polyline fill="none" stroke="#FFFFFF" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" points="
	                    0.0,75.0 37.5,37.5 0.0,0.0 "/>
                </svg>
                </button>
            </Link>;
        }
        return(
            <div className="preview_div">
                <Link className="close" to="/">
                    <svg width="20px" height="20px">
                        <line x1="2" y1="2" x2="20" y2="20" stroke="white" stroke-width="2" />
                        <line x1="20" y1="2" x2="2" y2="20" stroke="white" stroke-width="2" />
                    </svg>
                </Link>
                {prev}
                <img className="preview" src={`/assets/${this.props.entity.preview_path}`}/>
                {next}
            </div>);
    }
}

class MediaList extends React.Component {
    constructor(props) {
        super(props)
    }

    render() {
        let entities = [];
        if (this.props.entities != null) {
            for (let entity of this.props.entities) {
                entities.push(
                    <Link className="media-thumbnail" key={entity.id} to={`/media/${entity.id}`}>
                        <img src={`/assets/${entity.thumbnail_path}`} />
                    </Link>
                );
            }
        }
        return (
            <div className="media-thumbnail-list">
                {entities}
            </div>
        );
    }
}

class Media extends React.Component {
    constructor(props) {
        super(props)
        this.state = {
            entities: null,
        };
    }

    async getThumbnails() {
        this.setState({ entities: await Entities.fetch() });
    }

    componentDidMount() {
        this.getThumbnails();
    }

    render() {
        if(this.state.entities === null) {
            return "Loading";
        }
        const entities = this.state.entities.entity;
        return (
            <BrowserRouter>
                <Switch>
                    <Route exact path="/"><MediaList entities={entities} /></Route>
                    <Route exact path="/media/:id" children={({ match }) => {
                        for(let i = 0; i < entities.length; i++) {
                            const entity = entities[i];
                            if (entity.id != match.params.id) {
                                continue;
                            }
                            let prevEntity = null;
                            if (i > 0) {
                                prevEntity = entities[i-1];
                            }
                            let nextEntity = null;
                            if (i < entities.length-1) {
                                nextEntity = entities[i+1];
                            }
                            return(<Pic entity={entity} next-entity={nextEntity} prev-entity={prevEntity}/>);
                        }
                    }} />
                </Switch>
            </BrowserRouter>
        );
    }
}

class App extends React.Component {
    render() {
        return (
            <div className="content">
                <header>
                    <nav>
                        <a className="active" href="/">Media</a>
                        <a href="/">Tags</a>
                    </nav>
                    <Search options={["Europa", "Asien", "Asia", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "z", "aa", "bb", "cc", "dd", "ee", "ff", "gg", "hh", "ii", "jj", "kk", "ll", "mm", "nn", "oo", "pp", "qq", "rr", "ss", "tt", "uu", "vv", "ww", "zz"]} />
                </header>
                <Media />
            </div>
        );
    }
}

export default App;
