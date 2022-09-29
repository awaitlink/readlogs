addEventListener("fetch", (event) => {
    event.respondWith(handleRequest(event.request));
});

async function handleRequest(request) {
    const allowedOrigins = [
        "https://readlogs.pages.dev",
        "http://127.0.0.1:8080"
    ];

    const origin = request.headers.get("origin");

    if (!allowedOrigins.includes(origin)) {
        return notFound();
    }

    const { pathname, searchParams } = new URL(request.url);
    const version = searchParams.has("v") ? searchParams.get("v") : null;

    if (pathname.startsWith("/android")) {
        return await respond(origin, pathname, version, "android", "", true);
    }

    if (pathname.startsWith("/ios")) {
        return await respond(origin, pathname, version, "ios", ".zip", false);
    }

    if (pathname.startsWith("/desktop")) {
        return await respond(origin, pathname, version, "desktop", ".gz", true);
    }

    return notFound(origin);
}

async function respond(origin, pathname, version, platform, extension, isAGzip) {
    const regex = new RegExp("^[a-f0-9]{64}$");
    const key = pathname.split("/")[2];

    if (!regex.test(key)) {
        return notFound(origin);
    }

    var url = "https://debuglogs.org/" + key + extension;
    if (version !== null) {
        url = "https://debuglogs.org/" + platform + "/" + version + "/" + key + extension;
    }

    const response = await fetch(url);

    if (!response.ok || response.headers.get("content-length") === "243") { // Hacky, but this is the length that errors have.
        return notFound(origin);
    }

    const options = {
        status: response.status,
        headers: response.headers,
    };

    if (isAGzip) {
        options.encodeBody = "manual";
    }

    const newResponse = new Response(response.body, options);

    newResponse.headers.delete("cf-cache-status");
    newResponse.headers.delete("age");
    newResponse.headers.delete("last-modified");

    newResponse.headers.delete("x-cache");
    newResponse.headers.delete("via");
    newResponse.headers.delete("date");

    for (var pair of newResponse.headers.entries()) {
        if (pair[0].startsWith("x-amz")) {
            newResponse.headers.delete(pair[0]);
        }
    }

    setAllowOrigin(newResponse, origin);

    if (isAGzip) {
        newResponse.headers.set("content-encoding", "gzip");
    } else {
        newResponse.headers.set("content-type", "application/zip");
    }

    newResponse.headers.set("cloudflare-cdn-cache-control", "no-store");
    newResponse.headers.set("cache-control", "public, max-age=604800, immutable");

    return newResponse;
}

function notFound(origin) {
    let response = new Response("Not Found", { status: 404 });

    if (origin) {
        setAllowOrigin(response, origin);
    }

    return response;
}

function setAllowOrigin(response, origin) {
    response.headers.set("access-control-allow-origin", origin);
    response.headers.set("vary", "origin");
}
