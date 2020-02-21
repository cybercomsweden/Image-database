import React from "react";
import FilePreview from "react-preview-file";
import update from "immutability-helper";

import classes from "./css/upload.css";
import mediaClass from "./css/media-list.css";

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
                            <FilePreview file={file}>
                                {(preview) => (
                                    <span className={mediaClass.thumbnail}>
                                        <img src={preview} alt="No preview available" />
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
