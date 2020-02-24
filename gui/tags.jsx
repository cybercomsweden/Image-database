import update from "immutability-helper";
import React from "react";
import { Link } from "react-router-dom";
import { Tag, Tags as ApiTags } from "./api.js";

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
            addTagParentValue: "0",
            addClicked: false,
        };

        this.handleAddChange = this.handleAddChange.bind(this);
        this.handleAddSubmit = this.handleAddSubmit.bind(this);
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

    getAllTagNames() {
        const { tags: { tag: tags } } = this.state;
        const tagNames = [];
        if (!tags.length) {
            return null;
        }
        tagNames.push(<option key="0" value="0">None</option>);
        for (const tag of tags) {
            tagNames.push(
                <option key={tag.id} value={tag.id}>
                    {tag.name}
                </option>,
            );
        }

        return tagNames;
    }

    handleAddChange(event) {
        const { target } = event;
        if (target.name === "tagName") {
            this.setState({ addTagValue: target.value });
        } else if (target.name === "tagParent") {
            this.setState({ addTagParentValue: target.value });
        }
    }

    async handleAddSubmit(event) {
        event.preventDefault();
        const { tags: { tag: tags }, addTagValue, addTagParentValue } = this.state;
        const tagNames = [];
        for (const tag of tags) {
            tagNames.push(tag.canonical_name);
        }
        if (!tagNames.includes(addTagValue)) {
            const tag = await Tag.add(addTagParentValue, addTagValue);
            const { tags: stateTags } = this.state;
            const newTags = update(stateTags, { tag: { $push: [tag] } });
            this.setState({ tags: newTags });
        } else {
            // eslint-disable-next-line no-alert
            alert("Could not add tag since it already exists!");
        }
    }

    handleAddClick() {
        this.setState({ addClicked: true });
    }

    render() {
        const {
            tags: tagsPb, addClicked, addTagParentValue, addTagValue,
        } = this.state;
        let tagsResult;
        let allTags;
        if (tagsPb === null) {
            tagsResult = "Loading";
        }
        if (tagsPb !== null && !tagsPb.tag.length) {
            tagsResult = "No tags yet";
        }
        if (tagsPb !== null && tagsPb.tag.length) {
            tagsResult = this.getSubTree(0);
            allTags = this.getAllTagNames();
        }

        let addForm = null;
        if (addClicked) {
            addForm = (
                <form onSubmit={this.handleAddSubmit}>
                    <div className={classes.selectWrapper}>
                        <select className={classes.select} name="tagParent" value={addTagParentValue} onChange={this.handleAddChange}>
                            {allTags}
                        </select>
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
