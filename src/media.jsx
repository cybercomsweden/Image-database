import Pbf from 'pbf';
import React from 'react';
import { BrowserRouter, Switch, Route, Link, useParams } from 'react-router-dom';
import Search from './search.js';
import {Entity, Entities} from './entity.proto';

async function getThumbnailPaths() {
    const rsp = await fetch("/list");
    const blob = await rsp.blob();
    const buf = await blob.arrayBuffer();

    const thumbnailPaths = [];
    const entities = Entities.read(new Pbf(buf));
    for (let entity of entities.entity) {
        thumbnailPaths.push(entity);
    }

    return thumbnailPaths;
}

function getEntity(id) {
    var fetchPath = `/media/id/${id}`;
    const response = fetch(fetchPath).then((response) => {
        return response.blob();
    }).then((blob) => {
        return blob.arrayBuffer();
    }).then((buf) => {
        console.log(buf);
        return Entity.read(new Pbf(buf)).preview_path;
    });
    return response;
}

class Pic extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            entity: [],
            path: []
        };
    }

    getPath(id) {
        getEntity(id).then((path) => {
            let real_path = `/media/${path.replace(/\\/, "/")}`;
            this.setState({path: real_path});
        })
    }

    componentDidMount() {
        this.getPath(this.props.entity);
    }

    render() {
        return (
            <div>
                <img src={this.state.path} />
            </div>
        );
    }
}

class Media extends React.Component {
    constructor(props) {
        super(props)
        this.state = {
            media: [],
            entities: []
        };
    }

    getThumbnails() {
        getThumbnailPaths().then((entities) => {
            const thumbnails = [];
            var id = 0;
            for (let e of entities) {
                let orig_path = e.thumbnail_path;
                let path = `/media/${orig_path.replace(/\\/, "/")}`;
                let link = `/media/id/${e.id}`
                thumbnails.push(
                        <div key={id} className="media-thumbnail">
                            <Link to={link}><img src={path} /></Link>
                        </div>)
                id = id + 1;
            }

            this.setState({ media: thumbnails, entities: entities });
        });
    }

    componentDidMount() {
        this.getThumbnails();
    }

    render() {
        return (
            <div className="media-thumbnail-list">
                <BrowserRouter>
                    <Switch>
                        <Route exact path="/">
                            {this.state.media}
                        </Route>
                        <Route exact path="/media/id/:id" children={({ match }) => {
                            return <Pic entity={match.params.id} />;
                        }} />
                    </Switch>
                </BrowserRouter>
            </div>
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
                    <Search options={["Europa", "Asien", "Asia"]} />
                </header>
                <Media />
            </div>
        );
    }
}

export default App;
