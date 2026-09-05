type __VLS_Props = {
    /** The module's bytes, or a URL to fetch them from. */
    wasm: BufferSource | string;
    /** Where `phone-server` listens. */
    endpoint?: string;
    /** What this phone calls itself in the model. */
    label?: string;
    /** `RTCPeerConnection` configuration — `{iceServers}` for STUN. */
    configuration?: RTCConfiguration;
    /** What the dial field starts with. A host that knows where this phone calls sets it. */
    destination?: string;
};
declare const __VLS_export: import("vue").DefineComponent<__VLS_Props, {
    dial: () => Promise<void>;
    ready: Promise<void>;
}, {}, {}, {}, import("vue").ComponentOptionsMixin, import("vue").ComponentOptionsMixin, {}, string, import("vue").PublicProps, Readonly<__VLS_Props> & Readonly<{}>, {
    label: string;
    endpoint: string;
    destination: string;
    configuration: RTCConfiguration;
}, {}, {}, {}, string, import("vue").ComponentProvideOptions, false, {}, any>;
declare const _default: typeof __VLS_export;
export default _default;
