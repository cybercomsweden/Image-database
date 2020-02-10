import React from "react";
import { Tags as ApiTags } from "./api.js";

export class Tags extends React.Component {
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
        this.setState({ tags: await ApiTags.fetch() });
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
