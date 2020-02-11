import React from "react";
import { Link } from "react-router-dom";
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
            const dest = "media?q=".concat(tag.canonical_name);
            tags.push(
                <Link key={tag.canonical_name} to={dest}>
                    <div>
                        {tag.canonical_name}
                    </div>
                </Link>,
            );
        }
        return (
            <div className="tag-list">
                {tags}
            </div>
        );
    }
}
