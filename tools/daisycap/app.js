(() => {
  const root = document.documentElement;
  root.classList.add('js');

  const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const canObserve = 'IntersectionObserver' in window;

  // Entrances: each element animates once, when it first scrolls into view. The stagger delay is
  // cleared afterwards, so a hover transition never waits for it.
  const items = document.querySelectorAll('[data-reveal]');
  items.forEach((el) => el.style.setProperty('--d', `${el.dataset.delay || 0}ms`));
  const show = (el) => {
    el.classList.add('is-in');
    el.addEventListener('transitionend', () => el.style.setProperty('--d', '0ms'), { once: true });
  };
  if (reduce || !canObserve) {
    items.forEach((el) => el.classList.add('is-in'));
  } else {
    const reveal = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          show(entry.target);
          reveal.unobserve(entry.target);
        }
      });
    }, { threshold: 0.1 });
    items.forEach((el) => reveal.observe(el));
  }

  // The cut replay loops while it is on screen and stops when it scrolls away. The button restarts it.
  const replay = document.querySelector('[data-replay]');
  if (replay && !reduce) {
    const track = replay.querySelector('.track');
    const gap = replay.querySelector('.gap');
    // offsetWidth ignores transforms, so the gap's closed state still measures.
    const fit = () => track.style.setProperty('--shift', `-${gap.offsetWidth}px`);
    fit();
    window.addEventListener('resize', fit);

    let timer = null;
    let visible = false;
    const restart = () => {
      clearTimeout(timer);
      replay.classList.remove('play');
      void replay.offsetWidth;
      replay.classList.add('play');
      timer = setTimeout(() => { if (visible) restart(); }, 2200);
    };
    const stop = () => {
      clearTimeout(timer);
      replay.classList.remove('play');
    };
    replay.querySelector('.replay-btn').addEventListener('click', restart);

    if (canObserve) {
      const watch = new IntersectionObserver((entries) => {
        visible = entries[0].isIntersecting;
        if (visible) restart(); else stop();
      }, { threshold: 0.6 });
      watch.observe(replay);
    }
  }
})();
