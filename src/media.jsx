import Pbf from 'pbf';
import React from 'react';
import { BrowserRouter, Switch, Route, Link, useParams } from 'react-router-dom';
import Search from './search.js';
import {Entity, Entities} from './api.js';

class Pic extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            path: []
        };
    }

    async getPath(id) {
        // TODO: Update this to preview_path
        const entity = await Entity.fetch(id);
        this.setState({path: `/media/${entity.path}`});
    }

    componentDidMount() {
        this.getPath(this.props.entity);
    }

    render() {
        return <img src={this.state.path} />;
    }
}

class MediaList extends React.Component {
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
        let entities = [];
        if (this.state.entities != null) {
            for (let entity of this.state.entities.entity) {
                entities.push(
                    <Link className="media-thumbnail" key={entity.id} to={`/media/${entity.id}`}>
                        <img src={`/media/${entity.thumbnail_path}`} />
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

function Media(props) {
    return (
        <BrowserRouter>
            <Switch>
                <Route exact path="/"><MediaList /></Route>
                <Route exact path="/media/:id" children={({ match }) => {
                    return <Pic entity={match.params.id} />;
                }} />
            </Switch>
        </BrowserRouter>
    );
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
                    <Search options={["Europa", "Asien", "Asia"]} />
                </header>
                <Media />
            </div>
        );
    }
}

export default App;
