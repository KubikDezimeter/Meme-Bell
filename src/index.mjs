import { h, Component, render } from 'https://esm.sh/preact';
import htm from 'https://esm.sh/htm';
import { useState, useEffect } from 'https://esm.sh/preact/hooks';
import { signal } from 'https://esm.sh/@preact/signals';

// Initialize htm with Preact
const html = htm.bind(h);


const ringtoneList = signal([]);

function RingtoneList() {

    useEffect(async () => {
        ringtoneList.value = await fetch("/api/ringtone_list")
            .then((response) => response.json())
    }, []);

    return html`
    <table>
        <tr>
<!--            <th>Klingelton aktiv</th>-->
            <th>Name</th>
            <th></th>
            <th></th>
        </tr>
        ${ringtoneList.value.map((ringtone) => html`
        <tr key="${ringtone}">
<!--            <td><input type="checkbox" name="${ringtone}" /></td>-->
            <td>${ringtone}</td>
            <td><audio controls><source src="/api/ringtone/${ringtone}" type="audio/mpeg" />Your browser does not support the audio element.</audio></td>
            <td><${RemoveButton} ringtone="${ringtone}" /></td>
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

                await result;
                ringtoneList.value = [...ringtoneList.value, file.name];
                console.log(ringtoneList.value);

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

function RemoveButton(props) {
    const handleRemove = async () => {
        console.log("Moving ringtone " + props.ringtone + " to trash...");

        try {
            const result = await fetch("/api/remove/" + props.ringtone, {
                method: "POST",
            });
            ringtoneList.value = ringtoneList.value.filter((ringtone) => ringtone !== props.ringtone);
            console.log(ringtoneList.value)
        } catch (error) {
            console.error(error);
        }
    }

    return html`<button type="button" onclick=${handleRemove}>Entfernen</button>`;
}

function Settings() {
    return html`
        <${RingingTimeSetting} />
    `;
}

function RingingTimeSetting() {
    const [ringingTime, setRingingTime] = useState();
    useEffect(async () => {
        let value = await fetch("/api/settings/ringing_time")
            .then((response) => response.json());
        setRingingTime(value);
    }, []);

    const handleChange = async () => {
        let ringingTime = document.getElementById("ringingTime").value;
        try {
            const result = await fetch("/api/settings/ringing_time", {
                method: "PUT",
                body: ringingTime
            });
        } catch (error) {
            console.error(error);
        }
    }

    return html`
        <label for="ringingTime">Klingeldauer in Sekunden: </label>
        <input type="number" id="ringingTime" min="1" value=${ringingTime} oninput=${handleChange} />
    `;
}

function App () {
    return html`
        <h1>Meme Klingel</h1>
        <${RingtoneList} />
        <${RingtoneUploader} />
        <${Settings} />
    `;
}

render(html`<${App} />`, document.body);