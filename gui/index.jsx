/*
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
*/
import React from "react";
import ReactDOM from "react-dom";
import {
    BrowserRouter, NavLink, Route, Switch,
} from "react-router-dom";
import { Media } from "./media.jsx";
import { Search } from "./widgets/search.jsx";
import { Tags } from "./tags.jsx";
import { WorldMap } from "./world_map.jsx";
import { Upload } from "./upload.jsx";

import "./css/global.css";
import layout from "./css/layout.css";

function App() {
    const mediaIsActive = (_, { pathname }) => pathname.match(/^\/(media|$)/);
    return (
        <BrowserRouter>
            <div className="content">
                <header>
                    <nav className={layout.mainMenu}>
                        <NavLink to="/" isActive={mediaIsActive}>Media</NavLink>
                        <NavLink to="/tags">Tags</NavLink>
                        <NavLink to="/map">Map</NavLink>
                        <NavLink to="/upload">Upload</NavLink>
                    </nav>
                    <Search />
                </header>
                <Switch>
                    <Route path="/tags"><Tags /></Route>
                    <Route path="/map"><WorldMap /></Route>
                    <Route path="/upload"><Upload /></Route>
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
