import React from 'react';

export class Search extends React.Component {

    constructor(props) {
        super(props);
        this.state = {
            userInput: '',
            options: props.options,
            filteredOptions: props.options,
            showOptions: false,
            activeOption: 0,
            prevOption: 0,
        };
        this.onChange = this.onChange.bind(this);
        this.onKeyDown = this.onKeyDown.bind(this);
        this.onFocus = this.onFocus.bind(this);
        this.onBlur = this.onBlur.bind(this);
    }


    filterOptions(userData) {
        const matches = this.state.options.filter(
            (optionName) =>
                optionName.toLowerCase().indexOf(userData.toLowerCase()) > -1
        );
        return matches.filter(x => !this.state.userInput.split(' ').includes(x));

    }

    onChange(event) {
        const userInput = event.target.value;
        const userData = userInput.split(" ");
        const filteredOptions = this.filterOptions(userData[userData.length-1]);
        this.setState({
            userInput,
            filteredOptions,
            activeOption: 0,
            showOptions: true
        });
    }

    onKeyDown(event) {
        const { activeOption, filteredOptions } = this.state;
        if (event.key === 'Enter') {
            let newInput = this.state.userInput.split(' ');
            newInput[newInput.length-1] = filteredOptions[activeOption];
            let userInput = newInput.join(" ");
            this.setState({
                activeOption: 0,
                showOptions: false,
                userInput,
            });
        }
        else if (event.key === 'ArrowUp') {
            if (activeOption === 0) {
                return;
            }
            this.setState({ activeOption: activeOption - 1 });
        }
        else if (event.key === 'ArrowDown') {
            if (activeOption === filteredOptions.length - 1) {
                return;
            }
            this.setState({ activeOption: activeOption + 1 });
        }
    }

    onFocus(event) {
        this.setState({
            activeOption: this.state.prevOption,
            prevOption: 0,
            showOptions: true,
            });
    }
    onBlur(event) {
        this.setState({
            activeOption: 0,
            prevOption: this.state.activeOption,
            showOptions: false,
            });
    }

    render() {
        const {
            onChange,
            onClick,
            onKeyDown,

            state: {activeOption, filteredOptions, showOptions, userInput }
        } = this;
        let optionList;
        if (showOptions && filteredOptions.length) {
            optionList = (
                <ul className="options">
                    {filteredOptions.map((optionName, index) => {
                        let className;
                        if (index === activeOption) {
                            className = 'option-active';
                        }
                        return (
                            <li className={className} key={optionName}>
                                {optionName}
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
