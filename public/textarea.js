function waitForMarkdownElements() {
    const textarea = document.querySelector(".input-box");
    const footer = document.querySelector(".footer");

    if (textarea && footer) {
        initializeMarkdownEditor(textarea, footer);
    } else {
        setTimeout(waitForMarkdownElements, 100); // 必要な要素が見つかるまで再試行
    }
}

function initializeMarkdownEditor(textarea, footer) {
    const simplemde = new EasyMDE({
        element: textarea,
        minHeight: "25px",
        toolbar: [
            "bold", "italic", "heading", "|", "quote", "unordered-list", "ordered-list", "|", "preview", "|",
            {
                name: "addEntry",
                action: function () { addEntry(simplemde); },
                className: "fa fa-paper-plane",
                title: "Submit (Ctrl+Enter)"
            }
        ],
        autoDownloadFontAwesome: true,
        status: false,
        previewClass: ["editor-preview"],
    });
    
    // Ctrl+Enterでエントリー追加
    simplemde.codemirror.on("keydown", function (instance, event) {
        if (event.ctrlKey && event.key === "Enter") {
            event.preventDefault();
            addEntry(simplemde);
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

    // フッターの高さに合わせてエディタの高さを調整
    function resizeEditor() {
        const toolbarHeight = document.querySelector('.editor-toolbar').offsetHeight || 35; // ツールバーの高さ
        const footerHeight = footer.clientHeight;
        const newEditorHeight = footerHeight - toolbarHeight - 40; // 若干の余裕を持たせる
        simplemde.codemirror.getScrollerElement().style.height = `${newEditorHeight}px`;
        simplemde.codemirror.refresh(); // エディタの再描画
    }

    // 初期設定とリサイズイベント
    resizeEditor();
    window.addEventListener("resize", resizeEditor);
   
}

// Add Entry関数
function addEntry(simplemde) {
    const markdownContent = simplemde.value();
    window.send_update_and_add_entry(markdownContent); // Rust側に送信
    simplemde.value(""); // エディタをクリア
}

// ページの読み込み完了後にMarkdownエディタを初期化
document.addEventListener("DOMContentLoaded", waitForMarkdownElements);
