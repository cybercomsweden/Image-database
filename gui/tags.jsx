import React from "react";
import { Link } from "react-router-dom";
import { Tags as ApiTags } from "./api.js";

import classes from "./css/tag-list.css";

function cmp(a, b) {
    if (a < b) {
        return -1;
    }
    if (a > b) {
        return 1;
    }
    return 0;
}

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

    getSubTree(pid) {
        const { tags: { tag: tags } } = this.state;
        const children = tags.filter((tag) => tag.pid === pid);
        children.sort((tagA, tagB) => cmp(tagA.name.toLowerCase(), tagB.name.toLowerCase()));
        if (!children.length) {
            return null;
        }

        const childNodes = [];
        for (const child of children) {
            const dest = "/media?q=".concat(child.canonical_name);
            childNodes.push(
                <li>
                    <Link key={child.canonical_name} to={dest}>{child.name}</Link>
                    {this.getSubTree(child.id)}
                </li>,
            );
        }

        return <ul className={classes.tree}>{childNodes}</ul>;
    }

    render() {
        const { tags: tagsPb } = this.state;
        if (tagsPb === null) {
            return "Loading";
        }
        if (!tagsPb.tag.length) {
            return "No tags yet";
        }
        return this.getSubTree(0);
    }
}
