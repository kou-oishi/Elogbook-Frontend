function waitForMarkdownElements() {
    window.content = document.querySelector(".content");
    window.footer = document.querySelector(".footer");
    window.divider = document.querySelector(".resize-divider");
    window.filePreviews = document.getElementById("file-previews");
    window.textarea = document.querySelector(".input-box");


    if (window.content &&
        window.footer &&
        window.divider &&
        window.filePreviews &&
        window.textarea) {
        initializeMarkdownEditor();
    } else {
        setTimeout(waitForMarkdownElements, 100); // 必要な要素が見つかるまで再試行
    }
}

function initializeMarkdownEditor() {
    window.easyMDE = new EasyMDE({
        element: window.textarea,
        minHeight: "25px",
        toolbar: [
            "bold", "italic", "heading", "|",
            "quote", "unordered-list", "ordered-list", "|",
            "link", "image", "|",
            {
                name: "attachFile",
                action: function () { attachFile(); },
                className: "fa fa-file",
                title: "Attach Files"
            },
            "|", "preview", "|",
            {
                name: "addEntry",
                action: function () { addEntry(); },
                className: "fa fa-paper-plane",
                title: "Submit (Ctrl+Enter)"
            }
        ],
        autoDownloadFontAwesome: true,
        status: false,
        previewClass: ["editor-preview"],
        spellChecker: false
    });

    // Ctrl+Enterでエントリー追加
    window.easyMDE.codemirror.on("keydown", function (instance, event) {
        if (event.ctrlKey && event.key === "Enter") {
            event.preventDefault();
            addEntry();
        }
    });

    // Add Entryボタンの隣に「Add Entry」テキストを追加
    const addEntryButton = document.querySelector(".fa-paper-plane");
    if (addEntryButton) {
        const addEntryText = document.createElement("span");
        addEntryText.textContent = "Submit (Ctrl+Enter)";
        addEntryText.style.marginLeft = "5px"; // アイコンとテキストの間に余白
        addEntryButton.parentNode.appendChild(addEntryText);
    }

    // エディタ内のスタイルを設定
    window.easyMDE.codemirror.getWrapperElement().style.fontSize = "12px"; // 文字サイズを14pxに設定
    window.easyMDE.codemirror.getWrapperElement().style.lineHeight = "1"; // 行間を設定
}


async function showFilePreview(file, fileNumber) {
    const previewsContainer = document.getElementById("file-previews");

    const previewDiv = document.createElement("div");
    previewDiv.classList.add("file-preview");

    // サムネイル用のspan
    const thumbnailSpan = document.createElement("span");
    thumbnailSpan.classList.add("thumbnail-span");

    // 「×」ボタン
    const closeButton = document.createElement("button");
    closeButton.textContent = "×";
    closeButton.classList.add("close-button");

    // ボタンのクリックイベントでファイルを削除
    closeButton.onclick = () => {
        fileList = fileList.filter(f => f !== file); // fileListからファイルを削除
        updateFilePreviews(); // プレビューを再描画
    };

    // 画像とPDFのサムネイル生成
    if (file.type.startsWith("image/")) {
        const img = document.createElement("img");
        img.src = URL.createObjectURL(file);
        img.classList.add("preview-image");
        thumbnailSpan.appendChild(img);
    } else if (file.type === "application/pdf") {
        const thumbnailUrl = await generatePDFThumbnail(file);
        const img = document.createElement("img");
        img.src = thumbnailUrl;
        img.classList.add("preview-image");
        thumbnailSpan.appendChild(img);
    } else if (file.type === "text/plain") {
        // テキストファイルの場合、Material Iconを表示
        thumbnailSpan.innerHTML = '<i class="material-icons preview-icon">description</i>';
    } else {
        // 不明なファイルの場合、汎用アイコンを表示
        thumbnailSpan.innerHTML = '<i class="material-icons preview-icon">insert_drive_file</i>';
    }

    previewDiv.appendChild(closeButton); // 「×」ボタンをプレビューに追加
    previewDiv.appendChild(thumbnailSpan); // サムネイル追加

    // ファイル情報用のspan
    const fileInfoSpan = document.createElement("span");
    fileInfoSpan.classList.add("file-info");

    const fileNumberSpan = document.createElement("span");
    fileNumberSpan.textContent = `[${fileNumber}]`;
    fileNumberSpan.classList.add("file-number");

    const fileName = document.createElement("span");
    fileName.textContent = file.name;
    fileName.classList.add("file-name");

    fileInfoSpan.appendChild(fileNumberSpan);
    fileInfoSpan.appendChild(fileName);

    previewDiv.appendChild(fileInfoSpan);
    previewsContainer.appendChild(previewDiv);
}

async function generatePDFThumbnail(file) {
    const pdfData = await file.arrayBuffer();
    const pdf = await pdfjsLib.getDocument({
        data: pdfData,
        cMapUrl: 'https://cdn.jsdelivr.net/npm/pdfjs-dist@2.10.377/cmaps/',
        cMapPacked: true // 圧縮されたCMapファイルを使用
    }).promise;
    const page = await pdf.getPage(1); // The first page

    const canvas = document.createElement("canvas");
    const viewport = page.getViewport({ scale: 0.5 });
    canvas.width = viewport.width;
    canvas.height = viewport.height;

    const context = canvas.getContext("2d", { willReadFrequently: true });
    await page.render({ canvasContext: context, viewport: viewport }).promise;
    return canvas.toDataURL("image/png"); // 画像データURLを返す
}

// File list
let fileList = []; // File list
// Update the file previews
async function updateFilePreviews() {
    window.filePreviews.innerHTML = "";
    for (let i = fileList.length - 1; i >= 0; i--) {
        await showFilePreview(fileList[i], i + 1);
    }
}

// Attach files 
async function attachFile() {
    const input = document.createElement("input");
    input.type = "file";
    input.multiple = true;

    input.onchange = async function (event) {
        const newFiles = Array.from(event.target.files);
        fileList = fileList.concat(newFiles);
        await updateFilePreviews();
    };
    // File selector
    input.click();
}

// Add Entry関数
function addEntry() {
    const markdownContent = window.easyMDE.value();

    // Attachments if exist
    let files = [];
    if (fileList) {
        files = fileList;
    }

    // Send to the Rust side
    window.send_add_entry(markdownContent, files);
    // Clear
    window.easyMDE.value("");
    window.filePreviews.innerHTML = "";
    fileList = [];
}

document.addEventListener("DOMContentLoaded", waitForMarkdownElements);