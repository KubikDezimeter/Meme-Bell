import { h, Component, render } from 'https://esm.sh/preact';
import htm from 'https://esm.sh/htm';
import { useState, useEffect } from 'https://esm.sh/preact/hooks';


// Initialize htm with Preact
const html = htm.bind(h);

// document.addEventListener("click", () => location.reload())

function RingtoneList() {
    const [ringtoneList, setRingtoneList] = useState([]);

    useEffect(async () => {
        await fetch("/api/ringtone_list")
            .then((response) => response.json())
            .then(setRingtoneList);
    }, []);

    return html`
    <table>
        <tr>
            <th>Klingelton aktiv</th>
            <th>Name</th>
            <th></th>
            <th></th>
        </tr>
        ${ringtoneList.map((ringtone) => html`
        <tr>
            <td><input type="checkbox" name="${ringtone}" /></td>
            <td>${ringtone}</td>
            <td><audio controls><source src="/api/ringtone/${ringtone}" type="audio/mpeg" />Your browser does not support the audio element.</audio></td>
            <td><button type="button">Entfernen</button></td>
        </tr>`)}
    </table>`;
}

function RingtoneUploader() {
    const [file, setFile] = useState(null);

    const handleFileChange = (e) => {
        if (e.target.files) {
            setFile(e.target.files[0]);
        }
    }

    const handleUpload = async () => {
        if (file) {
            console.log("Uploading file...");

            const formData = new FormData();
            formData.append("file", file);

            try {
                // You can write the URL of your server or any other endpoint used for file upload
                const result = await fetch("/api/upload", {
                    method: "POST",
                    body: formData,
                });

                const data = await result.json();

                console.log(data);
            } catch (error) {
                console.error(error);
            }
        }
    };

    return html`
    <div>
        <label htmlFor="file" className="sr-only">
            Choose a file
        </label>
        <input id="file" type="file" onChange=${handleFileChange} />
    </div>
    ${file && (html`
    <section>
        File details:
        <ul>
            <li>Name: ${file.name}</li>
            <li>Type: ${file.type}</li>
            <li>Size: ${file.size} bytes</li>
        </ul>
    </section>
    `)}

    ${file && html`<button onClick=${handleUpload}>Upload a file</button>`}`;
}

function App (props) {
    return html`
        <h1>Meme Klingel</h1>
        <${RingtoneList} />
        <${RingtoneUploader} />
    `;
}

render(html`<${App} />`, document.body);