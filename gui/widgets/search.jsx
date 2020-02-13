import React from "react";
import {
    withRouter,
} from "react-router-dom";
import queryString from "query-string";
import { AutocompleteTags } from "../api.js";

class InnerSearch extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            userInput: "",
            options: null,
            showOptions: false,
            activeOption: -1,
            prevOption: -1,
        };
        this.onChange = this.onChange.bind(this);
        this.onKeyDown = this.onKeyDown.bind(this);
        this.onFocus = this.onFocus.bind(this);
        this.onBlur = this.onBlur.bind(this);
        this.onMouseDown = this.onMouseDown.bind(this);
    }

    componentDidMount() {
        this.getTags();
        this.updateUserInput();
    }

    componentDidUpdate(prevProps) {
        const navPages = ["/", "/map", "/tags"];
        const { location } = this.props;
        if (location !== prevProps.location && navPages.includes(location.pathname)) {
            // eslint-disable-next-line react/no-did-update-set-state
            this.setState({ userInput: "" });
        } else {
            this.updateUserInput();
        }
    }

    onChange(event) {
        const userInput = event.target.value;
        this.setState({
            userInput,
        });
    }

    onMouseDown(event) {
        event.preventDefault();
        const { userInput } = this.state;
        const newInput = userInput.split(" ");
        newInput[newInput.length - 1] = event.target.dataset.canonicalName;
        this.setState({
            userInput: newInput.join(" "),
        });
    }

    onKeyDown(event) {
        const { activeOption, userInput } = this.state;
        let newInput = userInput.split(" ");
        const filteredOptions = this.filterOptions(newInput);
        const { history } = this.props;
        if (event.key === "Enter") {
            if (activeOption === -1) {
                if (newInput[newInput.length - 1] === "") {
                    newInput = newInput.slice(0, -1);
                }
                // Updating the url with the searched terms
                history.push("/media?q=".concat(newInput.join("+")));
            } else {
                newInput[newInput.length - 1] = filteredOptions[activeOption].canonical_name;
                newInput.push("");
            }
            this.setState({
                activeOption: -1,
                userInput: newInput.join(" "),
            });
        } else if (event.key === "ArrowUp") {
            if (activeOption === -1) {
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
            prevOption: -1,
            showOptions: true,
        });
    }

    onBlur() {
        const { activeOption } = this.state;
        this.setState({
            activeOption: -1,
            prevOption: activeOption,
            showOptions: false,
        });
    }

    async getTags() {
        const tags = await AutocompleteTags.fetch();
        this.setState({ options: tags.tag });
    }

    updateUserInput() {
        const { location } = this.props;
        const newQ = queryString.parse(location.search).q;
        const { showOptions, userInput } = this.state;
        if (newQ && newQ !== userInput) {
            if (!showOptions) {
                this.setState({ userInput: newQ });
            }
        }
    }

    filterOptions(userData) {
        const { options } = this.state;
        // TODO: Only filters on canonical name, will not work with åöä
        if (!options) {
            return [];
        }
        const matches = options.filter(
            (optionName) => optionName.canonical_name
                .indexOf(userData[userData.length - 1].toLowerCase()) > -1,
        );
        // Removes the tags that are already used from the list of suggested tags
        return matches.filter((x) => !userData.includes(x.canonical_name));
    }

    render() {
        const {
            activeOption, showOptions, userInput,
        } = this.state;
        let optionList;
        const filteredOptions = this.filterOptions(userInput.split(" "));
        if (showOptions && filteredOptions.length) {
            optionList = (
                <ul className="options" onMouseDown={this.onMouseDown}>
                    {filteredOptions.map((tag, index) => {
                        let className;
                        if (index === activeOption) {
                            className = "option-active";
                        }
                        return (
                            <li
                                className={className}
                                key={tag.canonical_name}
                                data-canonical-name={tag.canonical_name}
                            >
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

export const Search = withRouter(InnerSearch);
