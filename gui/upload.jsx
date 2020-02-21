import React from "react";
import FilePreview from "react-preview-file";
import update from "immutability-helper";

import classes from "./css/upload.css";
import mediaClass from "./css/media-list.css";

function ImageLogo() {
    const style = {
        stroke: "black", fill: "none", strokeWidth: "1.5px", strokeLinecap: "round", strokeLinejoin: "round",
    };
    const svgStyle = {
        height: "40px", width: "40px",
    };
    return (
        <svg viewBox="0 0 40 40" style={svgStyle}>
            <g transform="scale(1.6666666666666667,1.6666666666666667)">
                <path style={style} d="M22.477,21.75c0,0.828-0.672,1.5-1.5,1.5H3.023c-0.828,0-1.5-0.672-1.5-1.5V2.25c0-0.828,0.672-1.5,1.5-1.5h15 c0.391,0,0.767,0.153,1.047,0.426l2.955,2.883c0.289,0.282,0.452,0.67,0.452,1.074L22.477,21.75z M8.273,5.25 c1.243,0,2.25,1.007,2.25,2.25s-1.007,2.25-2.25,2.25s-2.25-1.007-2.25-2.25S7.03,5.25,8.273,5.25z M18.813,18.2l-3.925-5.888 c-0.309-0.465-0.937-0.591-1.402-0.282c-0.105,0.07-0.197,0.159-0.269,0.263l-2.691,3.845L8.858,14.8 c-0.436-0.349-1.072-0.279-1.421,0.157c-0.019,0.023-0.036,0.047-0.053,0.072L5.273,18.2" />
            </g>
        </svg>

    );
}

class ImageLoader extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            error: false,
        };

        this.onImageError = this.onImageError.bind(this);
    }

    onImageError() {
        this.setState({ error: true });
    }

    render() {
        const { error } = this.state;
        if (!error) {
            return (
                <img {...this.props} onError={this.onImageError} alt=" " />
            );
        }
        return (
            <ImageLogo />
        );
    }
}

export class Upload extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            uploadProgress: [],
            highlight: false,
            draggedImages: [],
        };

        this.dropArea = React.createRef();
        this.progressBar = React.createRef();
        this.highlight = this.highlight.bind(this);
        this.unhighlight = this.unhighlight.bind(this);
        this.handleDrop = this.handleDrop.bind(this);
        this.uploadFile = this.uploadFile.bind(this);
    }

    uploadFile(file, i) {
        const url = "/media/upload";
        const xhr = new XMLHttpRequest();
        const formData = new FormData();
        xhr.open("POST", url, true);
        xhr.setRequestHeader("X-Requested-With", "XMLHttpRequest");

        // Update progress (can be used to show progress indicator)
        xhr.upload.addEventListener("progress", (e) => {
            const numerator = e.loaded * 100.0;
            this.updateProgress(i, (numerator / e.total) || 100);
        });
        xhr.addEventListener("readystatechange", () => {
            if (xhr.readyState === 4 && xhr.status === 200) {
                this.updateProgress(i, 100);
            } else if (xhr.readyState === 4 && xhr.status !== 200) {
                // Error. Inform the user
            }
        });

        formData.append("fileToUpload", file);
        xhr.send(formData);
    }

    handleFiles(filesParam) {
        const files = [...filesParam];
        const { draggedImages } = this.state;
        this.setState({ draggedImages: update(draggedImages, { $push: files }) });
        this.initializeProgress(files.length);
        files.forEach(this.uploadFile);
    }

    highlight(e) {
        e.preventDefault();
        this.setState({ highlight: true });
    }

    unhighlight(e) {
        e.preventDefault();
        this.setState({ highlight: false });
    }

    initializeProgress(numFiles) {
        // Needed to make sure that the upload progress
        // is reseted between each upload
        let uploadProgress = this.state;
        uploadProgress = [];
        this.progressBar.value = 0;

        for (let i = numFiles; i > 0; i -= 1) {
            uploadProgress.push(0);
        }
    }

    updateProgress(fileNumber, percent) {
        const { uploadProgress } = this.state;

        uploadProgress[fileNumber] = percent;
        const total = uploadProgress.reduce((tot, curr) => tot + curr, 0)
        / uploadProgress.length;
        this.progressBar.value = total;
    }

    handleDrop(e) {
        e.preventDefault();
        const dt = e.dataTransfer;
        const { files } = dt;

        this.unhighlight(e);
        this.handleFiles(files);
    }

    render() {
        const { highlight, draggedImages } = this.state;
        const style = {
            color: highlight ? "blue" : "inherit",
        };
        return (
        /* This empty <> is the React.Fragment object, esling removed the React.Fragment */
            <>
                <div
                    className={classes.dropArea}
                    id="drop-area"
                    style={style}
                    ref={(ref) => { this.dropArea = ref; }}
                    onDragEnter={this.highlight}
                    onDragOver={this.highlight}
                    onDragLeave={this.unhighlight}
                    onDrop={this.handleDrop}
                >
                    <p>
                        Upload one or multiple files by dragging and
                        dropping images within the dashed box
                    </p>
                    <progress ref={(ref) => { this.progressBar = ref; }} max="100" value="0" />
                </div>
                <div className={mediaClass.list}>
                    {
                        draggedImages.map((file) => (
                            <FilePreview file={file} key={file.toString()}>
                                {(preview) => (
                                    <span className={mediaClass.thumbnail}>
                                        <ImageLoader src={preview} alt="No preview available" />
                                    </span>
                                )}
                            </FilePreview>
                        ))
                    }
                </div>
            </>
        );
    }
}
