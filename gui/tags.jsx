Image database, allows the user to host a database themselves,
with the possibilities to tag and search after images.
Copyright (C) 2020 Cybercom group AB, Sweden
By Christoffer Dahl, Johanna Hultberg, Andreas Runfalk and Margareta Vi

Image database is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
import update from "immutability-helper";
import React from "react";
import { Link } from "react-router-dom";
import { Tag, Tags as ApiTags } from "./api.js";
import { SimpleSearch } from "./widgets/search.jsx";

import classes from "./css/tag-list.css";
import viewClasses from "./css/media-view.css";

function cmp(a, b) {
    if (a < b) {
        return -1;
    }
    if (a > b) {
        return 1;
    }
    return 0;
}

class AddButton extends React.Component {
    constructor(props) {
        super(props);
        this.handleClick = this.handleClick.bind(this);
    }

    handleClick() {
        const { onClick } = this.props;
        onClick();
    }

    render() {
        const { label } = this.props;
        return (
            <button type="button" className={classes.addButton} onClick={this.handleClick}>
                <svg className={classes.svgCircleplus} viewBox="0 0 100 100">
                    <circle cx="50" cy="50" r="45" fill="none" strokeWidth="5" />
                    <line x1="32.5" y1="50" x2="67.5" y2="50" strokeWidth="5" />
                    <line x1="50" y1="32.5" x2="50" y2="67.5" strokeWidth="5" />
                </svg>
                <span>
                    {label}
                </span>
            </button>
        );
    }
}

export class Tags extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            tags: null,
            addTagValue: "",
            addTagParentValue: { id: 0, name: "Add tag parent (optional)" },
            addClicked: false,
        };

        this.handleAddChange = this.handleAddChange.bind(this);
        this.handleAddSubmit = this.handleAddSubmit.bind(this);
        this.handleAddParent = this.handleAddParent.bind(this);
        this.handleAddClick = this.handleAddClick.bind(this);
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
                <li key={child.canonical_name}>
                    <Link key={child.canonical_name} to={dest}>{child.name}</Link>
                    {this.getSubTree(child.id)}
                </li>,
            );
        }

        return <ul className={classes.tree}>{childNodes}</ul>;
    }

    handleAddChange(event) {
        const { target } = event;
        if (target.name === "tagName") {
            this.setState({ addTagValue: target.value });
        }
    }

    async handleAddSubmit(event) {
        event.preventDefault();
        const { tags: { tag: tags }, addTagValue, addTagParentValue } = this.state;
        const tagNames = [];
        for (const tag of tags) {
            tagNames.push(tag.name);
            tagNames.push(tag.canonical_name);
        }
        if (!tagNames.includes(addTagValue)) {
            const tag = await Tag.add(addTagParentValue.id, addTagValue);
            const { tags: stateTags } = this.state;
            const newTags = update(stateTags, { tag: { $push: [tag] } });
            this.setState({ tags: newTags, addTagParentValue: { id: 0, name: "Add parent tag (optional)" }, addTagValue: "" });
        } else {
            // eslint-disable-next-line no-alert
            alert("Could not add tag since it already exists!");
        }
    }

    handleAddParent(tag) {
        this.setState({ addTagParentValue: tag });
    }

    handleAddClick() {
        const { addClicked: clicked } = this.state;
        this.setState({ addClicked: !clicked });
    }

    render() {
        const {
            tags: tagsPb, addClicked, addTagParentValue, addTagValue,
        } = this.state;
        let tagsResult;
        if (tagsPb === null) {
            tagsResult = "Loading";
        }
        if (tagsPb !== null && !tagsPb.tag.length) {
            tagsResult = "No tags yet";
        }
        if (tagsPb !== null && tagsPb.tag.length) {
            tagsResult = this.getSubTree(0);
        }

        let addForm = null;
        if (addClicked) {
            addForm = (
                <form onSubmit={this.handleAddSubmit}>
                    <div className={viewClasses.tagSearch} key="!search-input">
                        <SimpleSearch
                            className={classes.tagName}
                            placeholder={addTagParentValue.name}
                            onSelect={this.handleAddParent}
                        />
                    </div>
                    <input className={classes.tagName} type="text" name="tagName" value={addTagValue} onChange={this.handleAddChange} placeholder="Add tag" />
                    <input className={classes.addButton} type="submit" value="Submit" />
                </form>
            );
        }

        return (
            <div className="tags">
                {tagsResult}
                <div className={classes.addTags}>
                    <AddButton onClick={this.handleAddClick} label="Add new tag" />
                    {addForm}
                </div>
            </div>
        );
    }
}
