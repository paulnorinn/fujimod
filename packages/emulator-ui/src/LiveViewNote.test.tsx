import { render } from '@solidjs/web';
import { flush } from 'solid-js';
import { afterEach, expect, test } from 'vite-plus/test';
import { LiveViewNote } from './EmulatorWorkspace';

let dispose: (() => void) | undefined;
afterEach(() => {
  dispose?.();
  document.body.replaceChildren();
});

function mount(props: { live: string; liveViewSource: boolean }) {
  const host = document.createElement('div');
  document.body.append(host);
  dispose = render(() => <LiveViewNote {...props} />, host);
  flush();
  return host;
}

test('a live camera without a live-view source explains the blank screen', () => {
  const note = mount({ live: 'LIVE', liveViewSource: false }).querySelector('#live-view-note');
  expect(note?.textContent).toContain('No live-view image source');
});

test('a camera with a source, and a camera that is not live, show no note', () => {
  expect(mount({ live: 'LIVE', liveViewSource: true }).querySelector('#live-view-note')).toBeNull();
  dispose?.();
  dispose = undefined;
  document.body.replaceChildren();
  expect(
    mount({ live: 'STOPPED', liveViewSource: false }).querySelector('#live-view-note'),
  ).toBeNull();
});
