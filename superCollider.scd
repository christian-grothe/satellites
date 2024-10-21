(
s.boot;

t = TempoClock.new(120/60);

n = NetAddr.new("satellites.local", 8081);
w = {
    | path , b, n |
    var date, postfix, fullPath, headerFormat, msg;
    date = Date.getDate.format("%d%m%H%M%S").asString;
    headerFormat = "wav";
    postfix = "." ++ headerFormat;
    fullPath = path ++ date ++ postfix;
    fullPath.postln;
    b.write( fullPath, headerFormat: headerFormat );
};
 
b = Buffer.alloc(s, 44100 * 5, 1);

SynthDef(\rec,{
    | bufnum |
    var sig;
    sig = SoundIn.ar(1);
    RecordBuf.ar(sig,bufnum,loop:0, doneAction:2);
  }).add;
)


n.sendMsg("/test",123,"Hello");

(
  x = Synth(\rec, [\bufnum, b]);
  x.onFree({
    b.normalize;
    w.value("/home/christian/dev/satellites/recordings/", b, n);
    })
)

~minor = Scale.minor;

~seq = Pseq(~minor,inf).asStream;
~seq.next.midiratio;

~white = Pwhite(0.0,1.0).asStream;
~whiteB = Pwhite(0.01,0.1).asStream;
~rand = Pseq([0.25,0.5,1,2],inf).asStream;
~oct = Prand([0.5,1,2],inf).asStream;



(
  a.stop;
  a = Routine({
    {
      n.sendMsg("/sampler/play/next",
      \pitch, ~seq.next.midiratio * ~oct.next,
      \att, 0,
      \rel, 0.025,
      \hold,0,
      \gain, 1,
      \start, ~white.next,
      \length, ~whiteB.next,
      \bufIndex, (1..20).choose,
    );
      wait(0.025);
    }.loop;
  }).play(t);
)

a.stop;
a.free

(
  c.stop;
  c = Routine({
    {
      n.sendMsg("/sampler/play/next",
      \pitch, ~seq.next,
      \att, 0,
      \rel, [1].choose,
      \hold,[0].choose,
      \gain, 0.25,
      \start, [0.0].choose,
      \length, [0.5].choose,
      \bufIndex, [4].choose,
    );
      wait([0.5].choose);
    }.loop;
  }).play(t)
)

c.stop;
c.free;


(
  n.sendMsg("/sampler/play/next",
        \gain, 0.5,
        \pitch, [1].choose,
        \att, 0,
        \rel, [0.3].choose,
        \hold,[0].choose,
        \amp, 0.5,
        \start, [0.0].choose,
        \length, [1.0].choose,
        \bufIndex, 4,
  );
)

n.sendMsg("/synth/play");
