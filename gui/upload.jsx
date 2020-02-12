import React from "react";

export class Upload extends React.Component {
    static handleFiles(filesParam) {
        const files = [...filesParam];
        // this.initializeProgress(files.length);
        files.forEach(Upload.uploadFile);
        // files.forEach(this.previewFile);
    }

    /*
    static previewFile(file) {
        const reader = new FileReader();
        reader.readAsDataURL(file);
        reader.onload = function load() {
            const img = document.createElement("img");
            img.src = reader.result;
            document.getElementById("gallery").appendChild(img);
        };
    }
    */

    // TODO: This function is a static function for now,
    // when progress bar is fixed it shouldn't be static
    // it should also take an another argument i
    static uploadFile(file) {
        const url = "/media/upload";
        const xhr = new XMLHttpRequest();
        const formData = new FormData();
        xhr.open("POST", url, true);
        xhr.setRequestHeader("X-Requested-With", "XMLHttpRequest");

        /*
        // Update progress (can be used to show progress indicator)
        xhr.upload.addEventListener("progress", (e) => {
            this.updateProgress(i, (e.loaded * 100.0 / e.total) || 100);
        });
        xhr.addEventListener("readystatechange", () => {
            if (xhr.readyState === 4 && xhr.status === 200) {
                this.updateProgress(i, 100); // <- Add this
            } else if (xhr.readyState === 4 && xhr.status !== 200) {
                // Error. Inform the user
            }
        });

        */
        formData.append("fileToUpload", file);
        xhr.send(formData);
    }

    constructor(props) {
        super(props);
        this.state = {
            // uploadProgress: [],
            highlight: false,
        };

        this.dropArea = React.createRef();
        this.progressBar = React.createRef();
        this.highlight = this.highlight.bind(this);
        this.unhighlight = this.unhighlight.bind(this);
        this.handleDrop = this.handleDrop.bind(this);
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
        this.progressBar.value = 0;

        for (let i = numFiles; i > 0; i -= 1) {
            this.uploadProgress.push(0);
        }
    }

    // TODO: uncomment when fixing progressbar
    /*
    updateProgress(fileNumber, percent) {
        this.uploadProgress[fileNumber] = percent;
        const total = this.uploadProgress.reduce((tot, curr) => tot + curr, 0)
        / this.uploadProgress.length;
        this.progressBar.value = total;
    } */

    handleDrop(e) {
        e.persist();
        e.preventDefault();
        const dt = e.dataTransfer;
        const { files } = dt;

        this.unhighlight(e);
        Upload.handleFiles(files);
    }

    render() {
        const { highlight } = this.state;
        const style = {
            color: highlight ? "blue" : "inherit",
        };
        return (
            <div id="drop-area">
                <div
                    className="upload"
                    style={style}
                    ref={(ref) => { this.dropArea = ref; }}
                    onDragEnter={this.highlight}
                    onDragOver={this.highlight}
                    onDragLeave={this.unhighlight}
                    onDrop={this.handleDrop}
                >
                    <p className="p-upload">
                        Upload one or multiple files by dragging and
                        dropping images within the dashed box
                    </p>
                    <progress ref={(ref) => { this.progressBar = ref; }} max="100" value="0" className="progress-bar" />
                    <div id="gallery" />
                </div>
            </div>
        );
    }
}
