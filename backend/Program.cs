using System.Text.Json;
using System.Text.Json.Nodes;
using backend;
using FFmpeg.AutoGen;

var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();
app.UseFileServer();

InteropRust.Init();
var id = InteropRust.CreateTrack();

Task.Run(() =>
{
  unsafe
  {
    AVDictionary* dict = null;
    ffmpeg.av_dict_set(&dict, "rtsp_transport", "tcp", 0);
    ffmpeg.av_dict_set(&dict, "stimeout", "5000000", 0);
    
    var formatContext = ffmpeg.avformat_alloc_context();
    
    var result = ffmpeg.avformat_open_input(&formatContext, "", null, &dict);
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