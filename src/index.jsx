import React from 'react';
import ReactDOM from 'react-dom';

class Media extends React.Component {
    render() {
        let jsonObj = '{ "media": [ {"path": "1_resized.jpg"},{"path": "2_resized.jpg"},{"path": "3_resized.jpg"},{"path": "4_resized.jpg"}]}';
        let obj = JSON.parse(jsonObj);
        console.log(obj);
        const media = [];
        for (let entity of obj.media) {
            let path = `/media/dest/${entity.path}`;
            media.push(<div class="media-thumbnail"><img src={path}/></div>)
        }
        return (
            <div className="media-thumbnail-list">
            {media}
            </div>
        );
    }
}

ReactDOM.render(
    <Media />,
    document.getElementById('root')
)