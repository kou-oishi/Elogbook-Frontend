document.addEventListener("DOMContentLoaded", () => {
  // 要素が全て揃うまで待機してから初期化処理を実行
  waitForContentElement();
});

function waitForContentElement() {
  window.content = document.querySelector(".content");
  if (window.content) {
    //initializeAttachmentPreviews();
    observeContentChanges();
  } else {
    // 必要な要素が見つかるまで100ms間隔で再試行
    setTimeout(waitForContentElement, 100);
  }
}

function initializeAttachmentPreviews() {
  const attachmentElements = window.content.querySelectorAll(".attachment-preview, .text-attachment");
  attachmentElements.forEach(element => {
    const url = element.getAttribute("data-url");
    const id = element.getAttribute("data-id");

    if (element.classList.contains("text-attachment")) {
      fetchPreview(url, id, "text");
    }
  });
}

function observeContentChanges() {
  const observer = new MutationObserver(() => {
    initializeAttachmentPreviews();
  });
  const config = { childList: true, subtree: true };
  observer.observe(window.content, config);
}

async function fetchPreview(url, id, type) {
  try {
    const response = await fetch(url, { cache: "force-cache" });
    if (!response.ok) {
      throw new Error(`Failed to fetch preview for ${id}: ${response.statusText}`);
    }

    if (type === "text") {
      const text = await response.text();
      updateTextAttachment(id, text);
    } else if (type === "image") {
      console.log(`Image ${id} fetched successfully.`);
    }
  } catch (error) {
    console.error("Error fetching preview:", error);
  }
}

function updateTextAttachment(id, content) {
  const element = window.content.querySelector(`.text-attachment[data-id='${id}']`);
  if (element) {
    element.innerHTML = `<pre>${content}</pre>`;
  }
}
