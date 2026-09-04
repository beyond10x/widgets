// The softphone's outer representation — what a person looking at the device would see.
//
// `ess verify conform web` emits the player; this file is the one part it does not, and the one part
// that may know what this system is. Copied in beside the emitted files as `skin.js`, where the
// player looks for it. A specification without one gets the core panels and says so.
//
// It renders `softphone.presentation` — the domain that models the screen: `Console` with its four
// modes, `CallTile` per call, `Keypad` with a half-typed entry. It does **not** render
// `softphone.control.Call`, and the difference is the whole point of having both panels:
//
//   the call    is state       — what is true
//   the tile    is the UI      — what is on the screen
//   a view      is a projection — what a caller reading it would get, possibly late
//
// The three current scenarios issue no presentation command at all. A tile exists only because a
// binding fires `ShowCall`, which the scenario declares and never asserts — so the tile is drawn
// dashed. A phone that looked live when nothing asserted it would be the over-claim this whole page
// exists to avoid.

const { inject, computed } = await import('./assets/vue.esm-browser.prod.js')

// A skin brings its own styles. These lived in the emitted page for a while, which meant every
// specification carried CSS for a device only this one has.
const STYLE = `
  /* the skin */
  .phone{display:flex;gap:14px;flex-wrap:wrap}
  .bezel{width:190px;border:1px solid var(--edge);border-radius:16px;padding:9px;background:var(--bg)}
  .bar{font-family:var(--mono);font-size:9.5px;margin-bottom:6px;text-align:center}
  .reg.ok{color:var(--live)} .reg.no{color:var(--warn)} .reg.none{color:var(--dim)}
  .screen{min-height:150px;border:1px solid var(--edge);border-radius:9px;padding:8px;
          background:color-mix(in srgb,var(--panel) 60%,var(--bg))}
  .mode{font-family:var(--mono);font-size:10px;color:var(--live);text-align:center;margin-bottom:7px}
  .mode.absent{color:var(--dim);font-style:italic}
  .scr-empty{color:var(--dim);font-size:11px;text-align:center;margin-top:22px}
  .scr-empty .hint{font-size:10px;margin-top:5px}
  .tile{border:1px solid var(--live);border-radius:7px;padding:6px 7px;margin-bottom:5px}
  .tile.declared{border-style:dashed;border-color:var(--warn)}
  .tl-1{font-size:11.5px} .tl-2{font-family:var(--mono);font-size:10px;color:var(--dim)}
  .tl-3{font-size:9.5px;color:var(--warn);margin-top:3px}
  .entry{font-family:var(--mono);font-size:13px;text-align:right;margin-top:7px;color:var(--dim)}
  .pad{display:grid;grid-template-columns:repeat(3,1fr);gap:4px;margin-top:8px}
  .pad span{border:1px solid var(--edge);border-radius:6px;text-align:center;padding:5px 0;
            font-family:var(--mono);font-size:11px;color:var(--dim)}
  .beside{flex:1;min-width:190px}
  .callrow{display:flex;justify-content:space-between;gap:8px;padding:4px 0;border-bottom:1px solid var(--edge)}
  .mono{font-family:var(--mono);font-size:11px}
  .pill{font-family:var(--mono);font-size:10px;padding:1px 8px;border-radius:20px;
        background:color-mix(in srgb,var(--live) 18%,transparent);color:var(--live)}
`
if (!document.getElementById('skin-style')) {
  const tag = document.createElement('style')
  tag.id = 'skin-style'
  tag.textContent = STYLE
  document.head.append(tag)
}

export default {
  name: 'OuterSurface',
  setup() {
    const player = inject('player')
    const instancesOf = (domainSuffix) => computed(() =>
      Object.values(player.state.world.instances).filter((i) => i.entity.startsWith(`softphone.${domainSuffix}.`)))

    const tiles = computed(() => instancesOf('presentation').value.filter((i) => i.entity.endsWith('.CallTile')))
    const consoles = computed(() => instancesOf('presentation').value.filter((i) => i.entity.endsWith('.Console')))
    const keypads = computed(() => instancesOf('presentation').value.filter((i) => i.entity.endsWith('.Keypad')))
    const registrations = computed(() => instancesOf('sip').value.filter((i) => i.entity.endsWith('.Registration')))
    const calls = computed(() => Object.values(player.state.world.instances).filter((i) => i.entity === 'softphone.control.Call'))

    // The screen's mode. `Console` owns it; with no console instance there is nothing declaring a
    // mode, and inferring one from the call would be inventing UI state the model did not produce.
    const mode = computed(() => consoles.value[0]?.state ?? null)

    return { player, tiles, consoles, keypads, registrations, calls, mode }
  },
  template: `
  <div class="phone">
    <div class="bezel">
      <div class="bar">
        <span v-if="registrations.length" :class="['reg', registrations[0].state === 'Registered' ? 'ok' : 'no']">
          ● {{ registrations[0].state }}
        </span>
        <span v-else class="reg none">○ no registration — no scenario touches it</span>
      </div>

      <div class="screen">
        <div v-if="mode" class="mode">{{ mode }}</div>
        <div v-else class="mode absent" title="softphone.presentation.Console">no Console instance</div>

        <div v-if="!tiles.length" class="scr-empty">
          Nothing on screen.
          <div class="hint">A tile arrives when a binding fires <code>ShowCall</code>.</div>
        </div>
        <div v-for="t in tiles" :key="t.instance" class="tile" :class="{ declared: t.declared }">
          <div class="tl-1">{{ t.fields.contact_id || 'unknown caller' }}</div>
          <div class="tl-2">{{ t.state }}</div>
          <div v-if="t.declared" class="tl-3">declared by a binding · not asserted</div>
        </div>

        <div v-if="keypads.length" class="entry">{{ keypads[0].fields.entry || '—' }}</div>
      </div>

      <div class="pad">
        <span v-for="k in ['1','2','3','4','5','6','7','8','9','*','0','#']" :key="k">{{ k }}</span>
      </div>
    </div>

    <div class="beside">
      <h3>The call, which is not the screen</h3>
      <p v-if="!calls.length" class="empty">No call exists yet.</p>
      <div v-for="c in calls" :key="c.instance" class="callrow">
        <span class="mono">{{ c.fields.remote || c.instance }}</span>
        <span class="pill">{{ c.state }}</span>
      </div>
      <p class="footnote">
        <code>softphone.control.Call</code> is state. The tile above is
        <code>softphone.presentation.CallTile</code>, and it is the UI. They move separately, and a
        scenario that drives one does not drive the other.
      </p>
    </div>
  </div>`,
}
