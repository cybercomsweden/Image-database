import React from "react";
import { AutocompleteTags } from "./api.js";

export class Search extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            userInput: "",
            options: null,
            filteredOptions: null,
            showOptions: false,
            activeOption: 0,
            prevOption: 0,
        };
        this.onChange = this.onChange.bind(this);
        this.onKeyDown = this.onKeyDown.bind(this);
        this.onFocus = this.onFocus.bind(this);
        this.onBlur = this.onBlur.bind(this);
    }

    componentDidMount() {
        this.getTags();
    }

    onChange(event) {
        const userInput = event.target.value;
        const userData = userInput.split(" ");
        const filteredOptions = this.filterOptions(userData[userData.length - 1]);
        this.setState({
            userInput,
            filteredOptions,
            activeOption: 0,
            showOptions: true,
        });
    }

    onKeyDown(event) {
        const { activeOption, filteredOptions, userInput } = this.state;
        if (event.key === "Enter") {
            const newInput = userInput.split(" ");
            newInput[newInput.length - 1] = filteredOptions[activeOption].canonical_name;
            this.setState({
                activeOption: 0,
                showOptions: false,
                userInput: newInput.join(" "),
            });
        } else if (event.key === "ArrowUp") {
            if (activeOption === 0) {
                return;
            }
            this.setState({ activeOption: activeOption - 1 });
        } else if (event.key === "ArrowDown") {
            if (activeOption === filteredOptions.length - 1) {
                return;
            }
            this.setState({ activeOption: activeOption + 1 });
        }
    }

    onFocus() {
        const { prevOption } = this.state;
        this.setState({
            activeOption: prevOption,
            prevOption: 0,
            showOptions: true,
        });
    }

    onBlur() {
        const { activeOption } = this.state;
        this.setState({
            activeOption: 0,
            prevOption: activeOption,
            showOptions: false,
        });
    }

    async getTags() {
        const tags = await AutocompleteTags.fetch();
        this.setState({ options: tags.tag, filteredOptions: tags.tag });
    }

    filterOptions(userData) {
        const { options, userInput } = this.state;
        // TODO: Only filters on canonical name, will not work with åöä
        const matches = options.filter(
            (optionName) => optionName.canonical_name.indexOf(userData.toLowerCase()) > -1,
        );
        // Removes the tags that are already used from the list of suggested tags
        return matches.filter((x) => !userInput.split(" ").includes(x.canonical_name));
    }

    render() {
        const {
            activeOption, filteredOptions, showOptions, userInput,
        } = this.state;
        let optionList;
        if (showOptions && filteredOptions.length) {
            optionList = (
                <ul className="options">
                    {filteredOptions.map((tag, index) => {
                        let className;
                        if (index === activeOption) {
                            className = "option-active";
                        }
                        return (
                            <li className={className} key={tag.canonical_name}>
                                {tag.path.join("/")}
                            </li>
                        );
                    })}
                </ul>
            );
        }
        return (
            <div className="search-bar">
                <input
                    type="text"
                    className="search-field"
                    onChange={this.onChange}
                    onKeyDown={this.onKeyDown}
                    onClick={this.onClick}
                    onFocus={this.onFocus}
                    onBlur={this.onBlur}
                    value={userInput}
                    placeholder="Search"
                />
                {optionList}
            </div>
        );
    }
}

export default Search;
