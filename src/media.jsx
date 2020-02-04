import api from './entity_pb.js';
import React from 'react';
import { BrowserRouter, Switch, Route, Link, useParams } from 'react-router-dom'
import Search from './search.js';

function getThumbnailPaths() {
    const response = fetch("/list").then((response) => {
        return response.blob();
    }).then((blob) => {
        return blob.arrayBuffer();
    }).then((buf) => {
        const thumbnailPaths = [];
        for (let entity of api.Entities.deserializeBinary(buf).getEntityList()) {
            thumbnailPaths.push(entity);
        }

        return thumbnailPaths;
    });
    return response;
}

class Pic extends React.Component {
    constructor(props) {
        super(props);
        console.log(props);
        this.state = {
            entity: []
        };
    }

    render() {
        return (
            <div>
                {/* TODO: change hardcoded path to picture */}
                <img src="/media/dest/Thinker-Auguste-Rodin-Museum-Paris-1904.jpg" />
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
                let orig_path = e.getThumbnailPath();
                console.log(e.getId());
                let path = `/media/${orig_path.replace(/\\/, "/")}`;
                let link = `/media/${e.getId()}`
                thumbnails.push(
                        <div key={id} className="media-thumbnail">
                            <Link to={link}><img src={path} /></Link>
                        </div>)
                id = id + 1;
            }

            this.setState({ media: thumbnails, entities: entities });
            console.log(this.state.entities);
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
                        <Route exact path="/media/:id" children={({ match }) => {
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
