import api from './entity_pb.js';
import React from 'react';
import ReactDOM from 'react-dom';

function getThumbnailPaths() {
    const response = fetch("/list").then((response) => {
        return response.blob();
    }).then((blob) => {
        return blob.arrayBuffer();
    }).then((buf) => {
        const thumbnailPaths = [];
        for (let entity of api.Entities.deserializeBinary(buf).getEntityList()) {
            thumbnailPaths.push(entity.getThumbnailPath());
        }

        return thumbnailPaths;
    });
    return response;
}

class Media extends React.Component {
    constructor(props) {
        super(props)
        this.state = {
            media: []
        };
    }

    getThumbnails() {
        getThumbnailPaths().then((paths) => {
            const thumbnails = [];
            var id = 0;
            for (let p of paths) {
                let path = `/media/${p.replace(/\\/, "/")}`;
                thumbnails.push(<div key={id} className="media-thumbnail"><img src={path}/></div>);
                id = id + 1;
            }

            this.setState({ media: thumbnails });
        });
    }

    componentDidMount() {
        this.getThumbnails();
    }

    render() {
        return (
            <div className="media-thumbnail-list">
            {this.state.media}
            </div>
        );
    }
}

ReactDOM.render(
    <Media />,
    document.getElementById('root')
)