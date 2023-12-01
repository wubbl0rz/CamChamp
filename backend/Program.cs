using System.Text.Json;
using System.Text.Json.Nodes;
using backend;
using FFmpeg.AutoGen;

var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();
app.UseFileServer();

InteropRust.Init();
var id = InteropRust.CreateTrack();

//TODO: autodetect
ffmpeg.RootPath = "ffmpeg/lib";

ffmpeg.av_log_set_level(ffmpeg.AV_LOG_QUIET);

Task.Run(() =>
{
  return;
  unsafe
  {
    AVDictionary* dict = null;
    ffmpeg.av_dict_set(&dict, "rtsp_transport", "tcp", 0);
    ffmpeg.av_dict_set(&dict, "stimeout", "5000000", 0);
  
    var formatContext = ffmpeg.avformat_alloc_context();
  
    var result = ffmpeg.avformat_open_input(&formatContext, "rtsp://localhost:8554/mystream", null, &dict);

    result = ffmpeg.avformat_find_stream_info(formatContext, &dict);

    var stream = formatContext->GetStreams(AVMediaType.AVMEDIA_TYPE_VIDEO).First();
  
    var pkt = ffmpeg.av_packet_alloc();

    while (true)
    {
      //TODO: clear pkt ?
      result = ffmpeg.av_read_frame(formatContext, pkt);

      Console.WriteLine(result);

      var durationMs = pkt->duration * ((stream.time_base.num / (double)stream.time_base.den) * 1000);

      Console.WriteLine(pkt->buf->size);

      if (stream.index != pkt->stream_index)
        continue;

      Console.WriteLine(durationMs);

      InteropRust.SendFrame(id, durationMs, pkt->buf->size, (IntPtr)pkt->buf->data);

    }
  }
});

app.MapGet("/webrtc/start", () =>
{
  using var result = InteropRust.CreateConnection();

  Console.WriteLine("OFFER: ");
  Console.WriteLine(result.GetOffer());

  return Results.Json(new
  {
    sdp = result.GetOffer(),
    result.ClientId
  });
});

app.MapPost("/webrtc/answer", (JsonObject json) =>
{
  var id = json["id"]!.GetValue<uint>();
  var sdp = json["sdp"]!.GetValue<string>();
 
  Console.WriteLine("ANSWER:");
  Console.WriteLine(sdp);

  InteropRust.SetAnswer(id, sdp);
});

app.Run();