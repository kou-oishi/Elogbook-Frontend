window.addEventListener("DOMContentLoaded", waitForContentElement());

function waitForContentElement() {
  if (window.content) {
    initializeAttachmentPreviews();
    observeContentChanges();
  } else {
    setTimeout(waitForContentElement, 100);
  }
}

function observeContentChanges() {
  const observer = new MutationObserver(() => {
    initializeAttachmentPreviews();
  });
  const config = { childList: true, subtree: true };
  observer.observe(window.content, config);
}

function initializeAttachmentPreviews() {
  const attachmentElements = window.content.querySelectorAll(".attachment-preview, .text-attachment, .pdf-attachment");
  //console.log(attachmentElements);

  attachmentElements.forEach(element => {
    if (!element.dataset.initialized) {
      const url = element.getAttribute("data-url");
      const id = element.getAttribute("data-id");

      if (element.classList.contains("text-attachment")) {
        fetchPreview(url, id, "text");
      } else if (element.classList.contains("pdf-attachment")) {
        fetchPreview(url, id, "pdf");
      }
      element.dataset.initialized = true;
    }
  });
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
    } else if (type === "pdf") {
      const pdfBlob = await response.blob();
      const pdfUrl = URL.createObjectURL(pdfBlob);
      console.log("pdf URL: ", pdfUrl);
      updatePdfAttachment(id, pdfUrl);
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

function updatePdfAttachment(id, pdfUrl) {
  const element = window.content.querySelector(`.pdf-attachment[data-id='${id}']`);
  if (element) {
    element.innerHTML = `<iframe src="${pdfUrl}" type="application/pdf" width="100%" height="500px"></iframe>`;
  }
}

