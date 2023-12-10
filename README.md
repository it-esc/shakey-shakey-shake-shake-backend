This app listen on port 3000

# Path
`GET /IlllIllI`
return sse that emit event
```
{
    score_1: number,
    score_2: number,
    score_3: number,
    score_4: number
}
```

`POST /n` where n = 1, 2, 3, 4
add score to team n

`POST /reset`
Need `Authorzation: Bearer P_LEON_KHOD_THEP` headers
