#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_ROOT="$(cd -- "${SCRIPT_DIRECTORY}/../.." && pwd)"
readonly FRAMES_DIRECTORY="${REPOSITORY_ROOT}/target/apollo18/deploy/lunar-globe-frames"
readonly OUTPUT_PATH="${REPOSITORY_ROOT}/target/apollo18/deploy/lunar-globe.webp"
readonly FRAME_COUNT=300
readonly FRAMES_PER_SECOND=30

if ! command -v ffmpeg >/dev/null 2>&1; then
    echo "error: ffmpeg is required to encode the lunar globe animation" >&2
    exit 1
fi

rm -rf -- "${FRAMES_DIRECTORY}"
rm -f -- "${OUTPUT_PATH}"
(
    cd -- "${REPOSITORY_ROOT}"
    cargo run --release -p apollo18-native --bin lunar-globe -- \
        --fps "${FRAMES_PER_SECOND}" \
        --num-frames "${FRAME_COUNT}" \
        "${FRAMES_DIRECTORY}"
)

for ((frame_index = 0; frame_index < FRAME_COUNT; frame_index++)); do
    printf -v frame_name 'frame-%04d.png' "${frame_index}"
    if [[ ! -f "${FRAMES_DIRECTORY}/${frame_name}" ]]; then
        echo "error: missing lunar globe frame ${FRAMES_DIRECTORY}/${frame_name}" >&2
        echo "rerun: scripts/deploy/globe-webp-animation.sh" >&2
        exit 1
    fi
done

ffmpeg \
    -hide_banner \
    -y \
    -framerate "${FRAMES_PER_SECOND}" \
    -start_number 0 \
    -i "${FRAMES_DIRECTORY}/frame-%04d.png" \
    -frames:v "${FRAME_COUNT}" \
    -an \
    -c:v libwebp_anim \
    -lossless 1 \
    -compression_level 6 \
    -loop 0 \
    "${OUTPUT_PATH}"

echo "wrote ${OUTPUT_PATH}"
