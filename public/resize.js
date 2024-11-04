function waitForElements() {
    if (window.easyMDE) {
        // Initialise all the sizes
        resizeNow();
        // Event handlers
        initialiseDividerEvent();
        observeChanges();
    } else {
        // 必要な要素が見つかるまで100ms間隔で再試行
        setTimeout(waitForElements, 100);
    }
}

// Resize the file preview items
function resizePreviewItems() {
    const previews = document.querySelectorAll(".file-preview");
    const previewHeight = window.content.offsetHeight * 0.15; // contentの20%の高さ

    previews.forEach(preview => {
        preview.style.height = `${previewHeight}px`;

        // The icon size if applicable
        const thumbnailSpan = preview.querySelector(".thumbnail-span");
        const icon = thumbnailSpan ? thumbnailSpan.querySelector(".preview-icon") : null;
        if (icon) {
            const spanHeight = thumbnailSpan.clientHeight;
            icon.style.fontSize = `${spanHeight}px`;
        }
    });
}

// Resize all the items in the window depending on the divider's height
function resizeItems(newHeight) {
    window.content.style.height = `${window.innerHeight - newHeight}px`;
    window.footer.style.height = `${newHeight}px`;
    window.textarea.style.height = `${newHeight - 20}px`; // paddingを考慮

    // footerのサイズに合わせてfile-previewsのbottomを調整
    window.filePreviews.style.bottom = `${newHeight + 20}px`; // footerの高さ＋マージン

    // EasyMDEエディターの縦幅をfooterの高さに合わせて調整
    window.easyMDE.codemirror.getScrollerElement().style.height = `${newHeight - 50}px`; // paddingを考慮

    resizePreviewItems();
}

function resizeNow() {
    // The current divider's place
    const dividerPosition = window.divider.getBoundingClientRect();
    const newHeight = window.innerHeight - dividerPosition.top;
    resizeItems(newHeight);
}

function initialiseDividerEvent() {
    // Set the flag
    let isResizing = false;
    window.divider.addEventListener("mousedown", function () {
        isResizing = true;
        document.body.style.cursor = "ns-resize";
    });
    document.addEventListener("mouseup", function () {
        isResizing = false;
        document.body.style.cursor = "default";
    });

    document.addEventListener("mousemove", function (e) {
        if (!isResizing) return;
        // The place the mouse cursor is pointing
        const newHeight = window.innerHeight - e.clientY;
        resizeItems(newHeight);
    });

}

function observeChanges() {
    // The preview items should be resized when the window is resized
    window.addEventListener("resize", () => { resizePreviewItems(); });

    // Monitor any change of the content and file previews
    const observer = new MutationObserver(() => {
        resizePreviewItems();
    });
    const config = { childList: true, subtree: true, attributes: true, characterData: true };
    observer.observe(window.content, config);
    observer.observe(window.filePreviews, config);
}

// 初期化関数を呼び出して要素が見つかるまで待機
document.addEventListener("DOMContentLoaded", waitForElements);
