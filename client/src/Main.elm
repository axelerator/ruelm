port module Main exposing (main)

import Browser
import Generated.Bindings as B exposing (ToBackend, ToFrontend)
import Html exposing (Html, div, h1, text)
import Http exposing (Error)
import Json.Decode


port messageReceiver : (String -> msg) -> Sub msg


type alias SessionId =
    String


main : Program SessionId Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


type alias Model =
    { count : Int
    , sessionId : SessionId
    , messageFromBackend : String
    }


init : SessionId -> ( Model, Cmd Msg )
init sessionId =
    ( { count = 0, sessionId = sessionId, messageFromBackend = "" }
    , sendToBackend sessionId B.Connect
    )


sendToBackend : SessionId -> ToBackend -> Cmd Msg
sendToBackend sessionId msg =
    Http.post
        { url = "/send/" ++ sessionId
        , body = Http.jsonBody <| B.toBackendEncoder msg
        , expect = Http.expectWhatever SentToBackend
        }


type Msg
    = SentToBackend (Result Error ())
    | FromBackend String


updateFromBackend : ToFrontend -> Model -> ( Model, Cmd Msg )
updateFromBackend toFrontend model =
    case toFrontend of
        B.Welcome msg ->
            ( { model | messageFromBackend = msg }
            , Cmd.none
            )

        _ ->
            ( model, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        FromBackend jsonStr ->
            case Json.Decode.decodeString B.toFrontendDecoder jsonStr of
                Ok toFrontend ->
                    updateFromBackend toFrontend model

                _ ->
                    ( model
                    , Cmd.none
                    )

        SentToBackend _ ->
            ( model
            , Cmd.none
            )


subscriptions : Model -> Sub Msg
subscriptions _ =
    messageReceiver <| FromBackend


view : Model -> Html Msg
view model =
    div []
        [ h1 [] [ text model.messageFromBackend ]
        ]
