import React from "react";
import ReactDOM from "react-dom";
import {
    BrowserRouter, NavLink, Route, Switch,
} from "react-router-dom";
import { Media } from "./media.jsx";
import { Search } from "./widgets/search.jsx";
import { Tags } from "./tags.jsx";
import { WorldMap } from "./world_map.jsx";

import "./stylesheet.css";

function App() {
    const mediaIsActive = (_, { pathname }) => pathname.match(/^\/(media\/|$)/);
    return (
        <BrowserRouter>
            <div className="content">
                <header>
                    <nav>
                        <NavLink to="/" isActive={mediaIsActive}>Media</NavLink>
                        <NavLink to="/tags">Tags</NavLink>
                        <NavLink to="/map">Map</NavLink>
                    </nav>
                    <Search />
                </header>
                <Switch>
                    <Route path="/tags"><Tags /></Route>
                    <Route path="/map"><WorldMap /></Route>
                    <Route path="/"><Media /></Route>
                </Switch>
            </div>
        </BrowserRouter>
    );
}

ReactDOM.render(
    <App />,
    document.getElementById("root"),
);
