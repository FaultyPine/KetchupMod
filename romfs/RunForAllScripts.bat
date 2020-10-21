for %%a in (*) do (
	if NOT %%a==prcScript.exe (
		if NOT %%a==RunForAllScripts.bat (
			prcScript.exe %%a
		)
	)
)
